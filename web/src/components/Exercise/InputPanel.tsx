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
  Input,
  Field,
} from "@chakra-ui/react";

interface State {
	isVerifying: boolean;
	result: string;
	commandArgs: string;
}

const InputPanel: React.FC = () => {
	const {
		currentExercise,
		currentSession,
		updateExerciseSession,
		createExerciseSession
	} = useExercises();
	const [{ 
		isVerifying,
		result,
		commandArgs
	 }, setState] = useState<State>({
		isVerifying: false,
		result: "",
		commandArgs: "cargo run",
	});

	const onProjectFolderSelect = async () => {
		try {
			const projectFolder = await openFolderDialog();

			if(!projectFolder) {
				return;
			}

			if(currentSession) {
				await updateExerciseSession({
					id: currentSession.id,
					completedOn: null,
					commandArgs: commandArgs,
					folderPath: projectFolder
				})
			}
			else {
				await createExerciseSession({
					exerciseId: currentExercise.id,
					commandArgs: commandArgs,
					folderPath: projectFolder
				});
			}

		} catch (err) {
			console.error("Folder selection failed", err);

		}
	};

	const onVerify = async () => {
    	if (!currentSession) {
			return;
		}

		try {
			setState((prev) => ({ ...prev, isVerifying: true }));
			const response = await verifyExercise(currentSession.id);
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

	const onCommandArgs = async () => {
			if(currentSession) {
				await updateExerciseSession({
					id: currentSession.id,
					completedOn: null,
					commandArgs: commandArgs,
					folderPath: null
				})
			}
			else {
				await createExerciseSession({
					exerciseId: currentExercise.id,
					commandArgs: commandArgs,
					folderPath: ""
				});
			}
	}

	const onCommandArgsChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    	setState((prev) => ({ ...prev, commandArgs: event.target.value }));
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
			{currentSession?.folderPath ? `Change` : "Select project with solution"}
		</Button>

	  	<Field.Root>
          <Field.Label color="white">Run Arguments</Field.Label>
          <Input
            placeholder="Enter command arguments"
            value={commandArgs}
			onBlur={onCommandArgs}
			onChange={onCommandArgsChange}
            bg="gray.800"
            color="white"
            borderRadius="md"
          />
        </Field.Root>

		<Button
			onClick={onVerify}
			disabled={!currentSession?.folderPath || isVerifying}
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
