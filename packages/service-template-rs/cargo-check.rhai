// Post-generation hook: verify the emitted skeleton compiles.
// cargo-generate runs this in the newly created project directory.
let result = run("cargo check -p {{project-name}}-api 2>&1");
if result.exit_code != 0 {
    error("cargo check failed:\n" + result.stdout);
}
