use sp1_helper::{build_program_with_args, BuildArgs};

fn main() {
    #[allow(unused_mut)]
    let mut args: BuildArgs = Default::default();
    #[cfg(feature = "wrapped")]
    {
        args.features = vec!["wrapped".to_string()];
    }
    build_program_with_args("../circuits/sp1-step", args.clone());
    build_program_with_args("../circuits/sp1-committee", args.clone());
    build_program_with_args("../circuits/recursive-step", Default::default());
}
