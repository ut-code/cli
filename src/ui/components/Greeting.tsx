import { Box } from "ink";
import BigText from "ink-big-text";

export const Greeting = () => {
	return (
		<>
			<BigText text="We're" colors={["#ffffff", "#00ff87"]} />
			<Box>
				<BigText text="ut." />
				<BigText text="code" colors={["green"]} />
				<BigText text="();" />
			</Box>
		</>
	);
};
