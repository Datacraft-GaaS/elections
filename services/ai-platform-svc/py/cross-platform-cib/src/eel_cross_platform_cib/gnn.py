"""GraphSAGE model for CIB cluster prediction.

Implements GraphSAGE (Hamilton, Ying, Leskovec NeurIPS 2017) for node-level
binary classification: is this post node part of a CIB cluster?

The model takes a PyTorch Geometric homogeneous graph (post nodes only) with
node features derived from the heterogeneous graph (see corr.py for feature
extraction). It outputs a per-node probability of CIB membership.

Reference:
  Hamilton W., Ying Z., Leskovec J. (2017). Inductive Representation Learning on
  Large Graphs. NeurIPS 2017. https://arxiv.org/abs/1706.02216
"""

from __future__ import annotations

import torch
import torch.nn as nn
import torch.nn.functional as F

try:
    from torch_geometric.nn import SAGEConv

    _HAS_PYG = True
except ImportError:
    _HAS_PYG = False
    SAGEConv = None  # type: ignore[assignment,misc]


class GraphSAGECIB(nn.Module):
    """Two-layer GraphSAGE for CIB binary classification.

    Args:
        in_channels: Dimensionality of input node features.
        hidden_channels: Width of the hidden layer (default 64).
        out_channels: Number of output classes (2: non-CIB, CIB).
        dropout: Dropout probability applied after the first layer.
    """

    def __init__(
        self,
        in_channels: int,
        hidden_channels: int = 64,
        out_channels: int = 2,
        dropout: float = 0.3,
    ) -> None:
        super().__init__()
        if not _HAS_PYG or SAGEConv is None:
            raise ImportError(
                "torch_geometric is required for GraphSAGECIB. Install it with: pip install torch-geometric"
            )
        self.conv1 = SAGEConv(in_channels, hidden_channels)
        self.conv2 = SAGEConv(hidden_channels, out_channels)
        self.dropout = dropout

    def forward(self, x: torch.Tensor, edge_index: torch.Tensor) -> torch.Tensor:
        """Forward pass.

        Args:
            x: Node feature matrix of shape (N, in_channels).
            edge_index: Graph connectivity in COO format, shape (2, E).

        Returns:
            Log-softmax class scores of shape (N, out_channels).
        """
        x = self.conv1(x, edge_index)
        x = F.relu(x)
        x = F.dropout(x, p=self.dropout, training=self.training)
        x = self.conv2(x, edge_index)
        return F.log_softmax(x, dim=1)

    def predict_proba(self, x: torch.Tensor, edge_index: torch.Tensor) -> torch.Tensor:
        """Return per-node CIB membership probability (index 1 of softmax).

        Args:
            x: Node features (N, in_channels).
            edge_index: COO edge index (2, E).

        Returns:
            Tensor of shape (N,) with CIB probability in [0, 1].
        """
        self.eval()
        with torch.no_grad():
            log_probs = self.forward(x, edge_index)
            return torch.exp(log_probs)[:, 1]


def build_feature_matrix(
    posts_data: list[dict],
    feature_dim: int = 8,
) -> torch.Tensor:
    """Build a simple node feature matrix from post metadata.

    Features per node (post):
      0: log1p(likes)
      1: log1p(shares)
      2: log1p(comments)
      3: log1p(views)
      4: has_shared_from (0/1)
      5: platform_hash_norm (crc32(platform) % 1000 / 1000.0, stable across processes)
      6: hour_of_day_norm (0-1)
      7: engagement_ratio (shares / (likes + 1))

    Args:
        posts_data: List of dicts with keys matching Post fields.
        feature_dim: Must be >= 8.

    Returns:
        Float32 tensor of shape (N, feature_dim).
    """
    import math
    import zlib
    from datetime import datetime

    rows: list[list[float]] = []
    for pd in posts_data:
        stats = pd.get("engagement_stats", {})
        likes = stats.get("likes", 0) if isinstance(stats, dict) else 0
        shares = stats.get("shares", 0) if isinstance(stats, dict) else 0
        comments = stats.get("comments", 0) if isinstance(stats, dict) else 0
        views = stats.get("views", 0) if isinstance(stats, dict) else 0

        ts_raw = pd.get("ts", "")
        hour = 0.0
        if ts_raw:
            try:
                hour = datetime.fromisoformat(str(ts_raw)).hour / 23.0
            except (TypeError, ValueError):
                hour = 0.0  # unparseable timestamps contribute no hour signal

        platform = pd.get("platform", "")
        platform_norm = (zlib.crc32(platform.encode("utf-8")) % 1000) / 1000.0

        row = [
            math.log1p(likes),
            math.log1p(shares),
            math.log1p(comments),
            math.log1p(views),
            1.0 if pd.get("shared_from") else 0.0,
            platform_norm,
            hour,
            shares / (likes + 1),
        ]
        # Pad if feature_dim > 8.
        while len(row) < feature_dim:
            row.append(0.0)
        rows.append(row[:feature_dim])

    if not rows:
        return torch.zeros((0, feature_dim), dtype=torch.float32)
    return torch.tensor(rows, dtype=torch.float32)
