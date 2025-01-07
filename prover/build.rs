use sp1_helper::build_program_with_args;

fn main() {
    build_program_with_args("../circuits/sp1-step", Default::default());
    build_program_with_args("../circuits/sp1-step-wrapped", Default::default());
    build_program_with_args("../circuits/sp1-committee", Default::default());
    build_program_with_args("../circuits/sp1-committee-wrapped", Default::default());
}
