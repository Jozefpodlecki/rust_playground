import { createContext, useContext, useEffect, useState } from 'react';
import { getExercises } from './api';
import { Exercise } from './models';

export interface ExerciseState {
	current: Exercise | null;
	progressPercent: number;
	completedIds: string[];
	setCurrentId: (id: string) => void;
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
	const [currentId, setCurrentId] = useState<string | null>(null);
	const [completedIds, setCompletedIds] = useState<string[]>([]);

	useEffect(() => {

		getExercises();

	}, []);

	const verifyExercise = async (id: string): Promise<boolean> => {
		// const success = await invoke<boolean>("verify_exercise", { id });

		// if (success && !completedIds.includes(id)) {
		// 	setCompletedIds(prev => [...prev, id]);
		// }
		// return success;

		return Promise.resolve(true);
	};

	const progressPercent = (completedIds.length / 10) * 100;

	return (
	<ExerciseContext.Provider
		value={{ current: null, setCurrentId, completedIds, verifyExercise, progressPercent }}
	>
		{children}
	</ExerciseContext.Provider>
	);
	};