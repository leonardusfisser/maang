RULE OF THUMB
Project behavior (profiles) → Cargo.toml
Build environment (flags, env, mirrors) → .cargo/config.toml

🧩 Difference: Cargo.toml vs .cargo/config.toml
Aspect	
[profile.release] in root Cargo.toml	

[profile.release] in .cargo/config.toml
Scope	
Affects the workspace and all crates inside it	Affects every Cargo build that uses this directory as CWD (including sub-commands and tools)

Ownership	
Part of the project manifest — versioned, shared with other developers	Environment-specific — typically not published or versioned

Inheritance	
Merges across workspace members and overrides defaults	Overrides global compiler settings after manifest resolution

Best Use	
Declarative control of crate optimization, panic strategy, LTO, strip, etc.	Environment/CI tuning (extra flags, temp dirs, offline builds, mirrors)

Example 
Location	
/home/leon/maangframe/Cargo.toml	
/home/leon/maangframe/.cargo/config.toml

Propagation to Dependencies	Applies to all workspace crates	
Applies globally to all crates Cargo builds in that environment (even outside workspace if you reuse same $CARGO_HOME)

Version Control	Always committed	
Optional — often ignored or CI-specific

Priority	
Lower (Cargo applies these first)	
Higher (values here override manifest)