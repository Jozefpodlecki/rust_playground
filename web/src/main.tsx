import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { Provider } from "./components/ui/provider";
import { HashRouter } from "react-router-dom";
import { AppProvider } from "./appProvider";
import { ExerciseProvider } from "./ExerciseProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<Provider>
		<HashRouter>
			<React.StrictMode>
				<AppProvider>
					<ExerciseProvider>
						<App />
					</ExerciseProvider>
				</AppProvider>
			</React.StrictMode>	
		</HashRouter>
  </Provider>
);
