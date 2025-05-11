import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { Provider as ChakraProvider } from "./components/ui/provider";
import { HashRouter } from "react-router-dom";
import { AppProvider } from "./providers/AppProvider";
import { ExerciseProvider } from "./providers/ExerciseProvider";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<ChakraProvider>
		<HashRouter>
			<React.StrictMode>
				<AppProvider>
					<ExerciseProvider>
						<App />
					</ExerciseProvider>
				</AppProvider>
			</React.StrictMode>	
		</HashRouter>
  </ChakraProvider>
);
