# UnboundOS

Bare-metal x86_64 dataflow operating environment. The authoritative spec is
`docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`.

Current Codex workflow:

```bash
python3 scripts/status.py
python3 scripts/verify.py --mission current --dry-run
```

Mission state lives in `.codex/`; local Codex skills and review roles live in
`.agents/`.

The pinned Rust toolchain is installed with rustup. If the shell has not picked
up `/home/leah/.cargo/bin`, the repo scripts still resolve Cargo from that
standard rustup location.
