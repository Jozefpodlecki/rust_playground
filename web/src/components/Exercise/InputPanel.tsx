import { useState } from "react";
import { IconFolder, IconCheck } from "@tabler/icons-react";
import { createSession, openFolderDialog, updateSession, verifyExercise } from "@/api";
import { useExercises } from "@/providers/ExerciseProvider";
import {
  Box,
  Button,
  Icon,
  Text,
  VStack,
  Code,
} from "@chakra-ui/react";

interface State {
	projectFolder: string | null;
	isVerifying: boolean;
	result: string;
}

const InputPanel: React.FC = () => {
	const {
		currentExercise,
		currentSession,
		createExerciseSession
	} = useExercises();
	const [{ projectFolder, isVerifying, result }, setState] = useState<State>({
		projectFolder: currentSession?.folderPath || null,
		isVerifying: false,
		result: "",
	});

	const onProjectFolderSelect = async () => {
		try {
			const projectFolder = await openFolderDialog();

			if(!projectFolder) {
				return;
			}

			// if(sessionId) {
			// 	await updateSession({
			// 		id: sessionId,
			// 		exerciseId: exerciseId,
			// 		folderPath: projectFolder,
			// 		completedOn: null
			// 	})
			// }
			// else {
			// 	const id = await createSession({
			// 		exerciseId: exerciseId,
			// 		folderPath: projectFolder
			// 	})
			// }

			setState((state) => ({ ...state, projectFolder }));
		} catch (err) {
			console.error("Folder selection failed", err);

		}
	};

	const onVerify = async () => {
    	if (!projectFolder) {
			return;
		}

		try {
			setState((prev) => ({ ...prev, isVerifying: true }));
			const response = await verifyExercise(exercise.current!.id);
			setState((prev) => ({ ...prev, result: response, isVerifying: false }));
		} catch (error) {
			// setState((prev) => ({
			// 	...prev,
			// 	result: "Verification failed.",
			// 	verifying: false,
			// }));
			// console.error(error);
		}
	};

	return (
	<Box className="grow-1" bg="gray.900" color="white" p={6} borderRadius="xl" shadow="md">
		<VStack align="stretch">
		<Button
			onClick={onProjectFolderSelect}
			colorScheme="blue"
			variant="solid"
		>
			<IconFolder />
			{projectFolder ? `Change` : "Select project with solution"}
		</Button>

		<Button
			onClick={onVerify}
			disabled={!projectFolder || isVerifying}
			loading={isVerifying}
			loadingText="Verifying..."
			colorScheme="green"
			variant="solid"
		>
		<Icon as={IconCheck} />
			Verify
		</Button>

		{result && (
			<Text fontSize="sm" color="yellow.400">
			{result}
			</Text>
		)}
		</VStack>
	</Box>
  );
};

export default InputPanel;
