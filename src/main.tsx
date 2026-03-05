import { program } from "commander";
import { render } from "ink";
import { Logo } from "./ui/components/Logo";

program.action(() => {
	render(<Logo />);
});

program.parse();
