import { program } from "commander";
import { render } from "ink";
import { Greeting } from "./ui/components/Greeting";

program.action(() => {
	render(<Greeting />);
});

program.parse();
