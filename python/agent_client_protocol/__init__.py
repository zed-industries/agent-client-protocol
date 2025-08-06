"""
Agent Client Protocol Python Package

This package provides the Python implementation of the Agent Client Protocol (ACP),
a protocol that standardizes communication between code editors and coding agents.
"""

from .acp import (
    Client,
    Agent,
    RequestError,
    ClientSideConnection,
    AgentSideConnection,
)

__version__ = "0.0.1"

__all__ = [
    "Client",
    "Agent",
    "RequestError",
    "ClientSideConnection",
    "AgentSideConnection",
]
