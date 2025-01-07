use sp1_helper::{build_program_with_args, BuildArgs};

fn main() {
    let mut args: BuildArgs = Default::default();
    #[cfg(feature = "wrapped")]
    {
        args.features = vec!["wrapped".to_string()];
    }
    build_program_with_args("../circuits/sp1-step", args.clone());
    build_program_with_args("../circuits/sp1-committee", args);
}
