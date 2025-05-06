import { useState } from "react";

import "./App.css";
import { Flex, Spinner } from "@chakra-ui/react";

const App = () => {
 
	async function greet() {
		// setGreetMsg(await invoke("greet", { name }));
	}

	return (
	<main className="">
		
		<Flex height="100vh" align="center" justify="center">
			<Spinner size="lg" colorPalette="gray" />
		</Flex>
	</main>
	);
}

export default App;
