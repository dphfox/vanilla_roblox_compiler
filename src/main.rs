use anyhow::Result;

mod compile;
mod contempora;

mod vanilla;
mod roblox;

fn main() -> Result<()> {
	compile::do_icon_compile()
}
