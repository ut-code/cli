import { program } from "commander";
import { render } from "ink";
import { Counter } from "./ui/components/Logo";

program.action(() => {
	render(<Counter />);
});

program.parse();
