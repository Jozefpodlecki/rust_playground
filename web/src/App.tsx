import "./App.css";
import { Flex, Spinner } from "@chakra-ui/react";
import { Route, Routes } from "react-router-dom";
import Home from "./Home";
import { useApp } from "./providers/AppProvider";
import Exercise from "@/components/Exercise/Main";
import Example from "@/components/Example/Main";

const App: React.FC = () => {
	const app = useApp();
	
	return (
	<main className="">
		
		<Flex height="100vh" align="center" justify="center">
			{app.isLoading ? <Spinner size="lg" colorPalette="gray" /> :
			<Routes>
				<Route path="/" element={<Home />} />
				<Route path="/exercise" element={<Exercise />} />
				<Route path="/example" element={<Example />} />
			</Routes>}
		</Flex>
	</main>
	);
}

export default App;
