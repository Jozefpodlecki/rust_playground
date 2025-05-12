import { createContext, useContext, useEffect, useState } from 'react';
import { createSession, getExercises, getLastExerciseSession, updateSession } from '@/api';
import { CreateExerciseSession, Exercise, ExerciseSession, UpdateExerciseSession } from '@/models';

export interface ExerciseState {
	currentExercise: Exercise;
	currentSession: ExerciseSession | undefined;
	exercises: Exercise[];
	progressPercent: number;
	completedIds: string[];
	updateExerciseSession(payload: UpdateExerciseSession): Promise<ExerciseSession>;
	createExerciseSession(payload: CreateExerciseSession): Promise<ExerciseSession>;
	verifyExercise(id: string): Promise<boolean>;
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
		currentExercise: null!,
		currentSession: undefined,
		exercises: [],
		progressPercent: 0,
		completedIds: [],
		updateExerciseSession,
		createExerciseSession,
		verifyExercise
	});

	useEffect(() => {

		onLoad();

	}, []);

	async function onLoad() {

		try {
			const exercises = await getExercises();	
			const currentSession = await getLastExerciseSession();

			let exerciseId = exercises[0].id;
						
			if(currentSession) {
				exerciseId = currentSession.exerciseId;
			}
	
			const currentExercise = exercises.find(pr => pr.id === exerciseId)!;

			setState(pr => {
				return {
					...pr,
					currentExercise,
					currentSession,
					exercises
				}
			});
		} catch (error) {
			console.log(error);
		}
		
	}

	async function updateExerciseSession(payload: UpdateExerciseSession) {
		const session = await updateSession(payload);

		return session;
	}

	async function createExerciseSession(payload: CreateExerciseSession) {

		const session = await createSession(payload);

		setState(pr => {
			return {
				...pr,
				currentSession: session
			}
		})

		return session;
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