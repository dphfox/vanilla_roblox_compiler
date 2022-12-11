use anyhow::Result;

mod compile;
mod contempora;

mod vanilla;
mod roblox;

fn main() -> Result<()> {
	if false {
		compile::do_icon_compile()
	} else {
		contempora::do_lint_pass()
	}

}
