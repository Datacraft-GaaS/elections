// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use eel_abis_vendor::{
    FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
    IrisTemplate, MockVendor, ModalityRoutes, PythonVendor, Score, Template, VendorAdapter,
    VendorRouter,
};

// ---------------------------------------------------------------------------
// MockVendor tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_extract_fingerprint_returns_template() {
    let vendor = MockVendor;
    let image = FingerprintImage {
        data: b"fingerprint_pixels".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let tmpl = vendor.extract_fingerprint(&image).await.unwrap();
    assert_eq!(tmpl.vendor, "mock");
    assert!(tmpl.quality > 0.0);
    assert!(!tmpl.data.is_empty());
}

#[tokio::test]
async fn mock_extract_face_returns_embedding() {
    let vendor = MockVendor;
    let image = FaceImage {
        data: b"face_pixels".to_vec(),
        mime_type: "image/jpeg".into(),
    };
    let tmpl = vendor.extract_face(&image).await.unwrap();
    assert_eq!(tmpl.vendor, "mock");
    assert!(tmpl.embedding.is_some());
    let emb = tmpl.embedding.unwrap();
    assert!(!emb.is_empty());
    for v in &emb {
        assert!(
            *v >= 0.0 && *v <= 1.0,
            "embedding values should be in [0,1]"
        );
    }
}

#[tokio::test]
async fn mock_extract_iris_returns_template() {
    let vendor = MockVendor;
    let image = IrisImage {
        data: b"iris_pixels".to_vec(),
        width: 640,
        height: 480,
    };
    let tmpl = vendor.extract_iris(&image).await.unwrap();
    assert_eq!(tmpl.vendor, "mock");
    assert!(tmpl.quality > 0.0);
}

#[tokio::test]
async fn mock_match_1_1_identical_templates_score_one() {
    let vendor = MockVendor;
    let image = FingerprintImage {
        data: b"same_data".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let tmpl = vendor.extract_fingerprint(&image).await.unwrap();
    let probe = Template::Fingerprint(tmpl.clone());
    let gallery = Template::Fingerprint(tmpl);
    let score = vendor.match_1_1(&probe, &gallery).await.unwrap();
    assert!(
        (score.value - 1.0).abs() < 1e-6,
        "identical templates should score 1.0"
    );
}

#[tokio::test]
async fn mock_match_1_1_different_templates_score_low() {
    let vendor = MockVendor;
    let img_a = FingerprintImage {
        data: b"aaa".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let img_b = FingerprintImage {
        data: b"bbb".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let probe = Template::Fingerprint(vendor.extract_fingerprint(&img_a).await.unwrap());
    let gallery = Template::Fingerprint(vendor.extract_fingerprint(&img_b).await.unwrap());
    let score = vendor.match_1_1(&probe, &gallery).await.unwrap();
    assert!(
        score.value < 0.5,
        "different templates should score low, got {}",
        score.value
    );
}

#[tokio::test]
async fn mock_match_1_n_returns_one_score_per_gallery_entry() {
    let vendor = MockVendor;
    let img = FingerprintImage {
        data: b"probe".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let probe = Template::Fingerprint(vendor.extract_fingerprint(&img).await.unwrap());

    let gallery: Vec<Template> = vec![
        Template::Fingerprint(FingerprintTemplate {
            vendor: "mock".into(),
            version: 1,
            data: b"g1".to_vec(),
            quality: 0.8,
        }),
        Template::Fingerprint(FingerprintTemplate {
            vendor: "mock".into(),
            version: 1,
            data: b"g2".to_vec(),
            quality: 0.7,
        }),
    ];
    let scores = vendor.match_1_n(&probe, &gallery).await.unwrap();
    assert_eq!(scores.len(), 2);
}

#[tokio::test]
async fn mock_match_1_n_empty_gallery_returns_empty() {
    let vendor = MockVendor;
    let img = FingerprintImage {
        data: b"probe".to_vec(),
        width: 500,
        height: 500,
        dpi: 500,
    };
    let probe = Template::Fingerprint(vendor.extract_fingerprint(&img).await.unwrap());
    let scores = vendor.match_1_n(&probe, &[]).await.unwrap();
    assert!(scores.is_empty());
}

#[tokio::test]
async fn mock_health_returns_healthy() {
    let vendor = MockVendor;
    let status = vendor.health().await.unwrap();
    assert_eq!(status, HealthStatus::Healthy);
}

#[tokio::test]
async fn mock_capabilities_covers_all_modalities() {
    let vendor = MockVendor;
    let caps = vendor.capabilities();
    assert!(caps.fingerprint);
    assert!(caps.face);
    assert!(caps.iris);
}

// ---------------------------------------------------------------------------
// VendorRouter tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn router_delegates_fingerprint_to_primary() {
    let mock: Arc<dyn VendorAdapter> = Arc::new(MockVendor);
    let routes = ModalityRoutes {
        fingerprint: mock.clone(),
        face: mock.clone(),
        iris: mock.clone(),
    };
    let router = VendorRouter::new(routes);
    let image = FingerprintImage {
        data: b"pixels".to_vec(),
        width: 400,
        height: 400,
        dpi: 500,
    };
    let tmpl = router.extract_fingerprint(&image).await.unwrap();
    assert_eq!(tmpl.vendor, "mock");
}

#[tokio::test]
async fn router_health_all_healthy() {
    let mock: Arc<dyn VendorAdapter> = Arc::new(MockVendor);
    let routes = ModalityRoutes {
        fingerprint: mock.clone(),
        face: mock.clone(),
        iris: mock.clone(),
    };
    let router = VendorRouter::new(routes);
    let status = router.health().await.unwrap();
    assert_eq!(status, HealthStatus::Healthy);
}

// ---------------------------------------------------------------------------
// PythonVendor: health returns unhealthy when sidecar is not running
// ---------------------------------------------------------------------------

#[tokio::test]
async fn python_vendor_health_unreachable_returns_unhealthy() {
    // Use a port guaranteed to be closed in test environment.
    let vendor = PythonVendor::new("http://127.0.0.1:19999");
    let status = vendor.health().await.unwrap();
    assert_eq!(status, HealthStatus::Unhealthy);
}

// ---------------------------------------------------------------------------
// Score bounds
// ---------------------------------------------------------------------------

#[test]
fn score_clamps_to_range() {
    let s = Score::new(1.5);
    assert!((s.value - 1.0).abs() < 1e-6);
    let s2 = Score::new(-0.5);
    assert!((s2.value - 0.0).abs() < 1e-6);
}
