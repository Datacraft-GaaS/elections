package eel.loadtest

import io.gatling.core.Predef._
import io.gatling.http.Predef._
import scala.concurrent.duration._

/**
 * Gatling simulation: RTS + BVI combined at 5x load (WP-6.4).
 *
 * Targets:
 *   RTS: 300k concurrent; p95 <= 500ms, p99 <= 1000ms
 *   BVI: 90k req/s; p95 <= 200ms, p99 <= 500ms
 *
 * ADR-0018: write-path P99 must hold at <= 25ms under 5x load.
 *
 * Compile: sbt "Gatling/test" (requires sbt + Java 17+)
 * Skip: document in CI if sbt not installed.
 */
class FiveXSimulation extends Simulation {

  val baseUrl: String = sys.env.getOrElse("EEL_BASE_URL", "http://localhost:8080")
  val scalingFactor: Double = sys.env.getOrElse("EEL_SCALING_FACTOR", "5.0").toDouble

  // RTS: 300k concurrent at 5x; scaled for profile.
  val rtsTargetVUs: Int = math.max(1, (300000 * (scalingFactor / 5.0)).toInt)
  // BVI: 90k req/s at 5x.
  val bviTargetRps: Int = math.max(1, (90000 * (scalingFactor / 5.0)).toInt)

  val httpProtocol = http
    .baseUrl(baseUrl)
    .acceptHeader("application/json")
    .contentTypeHeader("application/json")
    .userAgentHeader("EelLoadTest-5x/1.0 (Gatling)")
    .shareConnections

  // Station and candidate feeders.
  val stationFeeder = Iterator.continually(
    (1 to 47000).map(i => Map("stationCode" -> s"STN-$i", "kitId" -> s"kit-5x-$i"))
  ).flatten

  val bviFeeder = Iterator.continually(
    (1 to 10000000).map(i => Map("enrolledId" -> s"enr-5x-$i", "kitId" -> s"kit-bvi5x-$i"))
  ).flatten

  val rtsScenario = scenario("RTS 34A submission at 5x")
    .feed(stationFeeder)
    .exec(
      http("POST RTS submit")
        .post("/rts/submit")
        .body(StringBody(session =>
          s"""{
            "kit_id":"${session("kitId").as[String]}",
            "station_code":"${session("stationCode").as[String]}",
            "form_type":"34A",
            "transmitted_at":"${java.time.Instant.now()}",
            "candidate_results":[
              {"candidate_id":"cand-001","votes":${(math.random() * 500).toInt}},
              {"candidate_id":"cand-002","votes":${(math.random() * 500).toInt}}
            ],
            "attestation_token":"test-attest.stub",
            "load_profile":"5x"
          }"""
        )).asJson
        .check(status.in(200, 202))
        .check(responseTimeInMillis.lt(1000))
    )
    .pause(50.milliseconds, 200.milliseconds)

  val bviScenario = scenario("BVI identification at 5x")
    .feed(bviFeeder)
    .exec(
      http("POST BVI identify")
        .post("/bvi/identify")
        .body(StringBody(session =>
          s"""{
            "enrolled_id":"${session("enrolledId").as[String]}",
            "kit_id":"${session("kitId").as[String]}",
            "station_code":"STN-${(math.random() * 47000).toInt}",
            "fingerprint_template":"${"aa" * 256}",
            "mode":"1:1",
            "load_profile":"5x"
          }"""
        )).asJson
        .check(status.is(200))
        .check(responseTimeInMillis.lt(500))
    )
    .pause(10.milliseconds, 50.milliseconds)

  setUp(
    // RTS: ramp to 300k concurrent over 5 min, hold 20 min.
    rtsScenario.inject(
      rampConcurrentUsers(0).to(rtsTargetVUs).during(5.minutes),
      constantConcurrentUsers(rtsTargetVUs).during(20.minutes),
      rampConcurrentUsers(rtsTargetVUs).to(0).during(5.minutes)
    ),
    // BVI: ramp to 90k req/s over 3 min, hold 15 min.
    bviScenario.inject(
      rampUsersPerSec(0).to(bviTargetRps).during(3.minutes),
      constantUsersPerSec(bviTargetRps).during(15.minutes),
      rampUsersPerSec(bviTargetRps).to(0).during(2.minutes)
    )
  ).protocols(httpProtocol)
    .assertions(
      // RTS p95 <= 500ms, p99 <= 1000ms.
      forAll.responseTime.percentile(95).lt(500),
      forAll.responseTime.percentile(99).lt(1000),
      // BVI-specific: check by scenario name.
      details("BVI identification at 5x").responseTime.percentile(95).lt(200),
      details("BVI identification at 5x").responseTime.percentile(99).lt(500),
      // Global error rate < 1%.
      global.failedRequests.percent.lt(1)
    )
}
