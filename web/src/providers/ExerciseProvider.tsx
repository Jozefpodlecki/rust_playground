import { createContext, useContext, useEffect, useState } from 'react';
import { getExercises, getLastExerciseSession } from '@/api';
import { Exercise, ExerciseSession } from '@/models';

export interface ExerciseState {
	current: ExerciseSession | undefined;
	exercises: Exercise[];
	progressPercent: number;
	completedIds: string[];
	verifyExercise: (id: string) => Promise<boolean>;
}

const ExerciseContext = createContext<ExerciseState | undefined>(undefined);

export const useExercises = () => {
	const context = useContext(ExerciseContext);
	if (!context)
		throw new Error("useExercises must be used within ExerciseProvider");

	return context;
};

export const ExerciseProvider: React.FC<React.PropsWithChildren> = ({ children }) => {
	const [state, setState] = useState<ExerciseState>({
		current: undefined,
		exercises: [],
		progressPercent: 0,
		completedIds: [],
		verifyExercise
	});

	useEffect(() => {

		onLoad();

	}, []);

	async function onLoad() {

		try {
			const exercises = await getExercises();	
			const session = await getLastExerciseSession();


			setState(pr => {
				return {
					...pr,
					current: session,
					exercises
				}
			});
		} catch (error) {
			console.log(error);
		}
		
	}

	async function verifyExercise(id: string): Promise<boolean> {
		// const success = await invoke<boolean>("verify_exercise", { id });

		// if (success && !completedIds.includes(id)) {
		// 	setCompletedIds(prev => [...prev, id]);
		// }
		// return success;

		return Promise.resolve(true);
	};

	return (
	<ExerciseContext.Provider
		value={state}
	>
		{children}
	</ExerciseContext.Provider>
	);
	};