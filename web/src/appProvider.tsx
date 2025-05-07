import { useContext, createContext, PropsWithChildren, useEffect, useState } from "react";
import { load } from "./api";

const defaultState = {
	appName: "",
    rustVersion: "",
    githubLink: "",
    version: "",
	isLoading: true,
	loadedOn: "",
}

const AppContext = createContext<AppState>(defaultState);

export function useApp(): AppState {
	return useContext<AppState>(AppContext);
}

export interface AppState {
	appName: string,
    rustVersion: string,
    githubLink: string,
    version: string,
	isLoading: boolean;
	loadedOn: string;
}

export const AppProvider: React.FC<PropsWithChildren> = ({ children }) => {
	const [state, setState] = useState<AppState>(defaultState);
	
	useEffect(() => {
		onLoad()

		return () => {

		}
	}, []);

	async function onLoad() {
		const result = await load();

		setState(state => ({
			...state,
			isLoading: false,
			...result
		}))
	}

	return (
	<AppContext.Provider value={state}>
		{children}
	</AppContext.Provider>
	);
};