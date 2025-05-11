import { useState } from "react";
import { IconFolder, IconCheck } from "@tabler/icons-react";
import { openFolderDialog, verifyExercise } from "@/api";
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
	projectFolder: string;
	verifying: boolean;
	result: string;
}

const InputPanel: React.FC = () => {
	const [{ projectFolder, verifying, result }, setState] = useState<State>({
		projectFolder: "",
		verifying: false,
		result: "",
	});

	const exercise = useExercises();

	const onProjectFolderSelect = async () => {
	try {
		const projectFolder = await openFolderDialog();
		debugger;
		if(!projectFolder) {
			return;
		}

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
			setState((prev) => ({ ...prev, verifying: true }));
			const response = await verifyExercise(exercise.current!.id);
			setState((prev) => ({ ...prev, result: response, verifying: false }));
		} catch (err) {
			setState((prev) => ({
				...prev,
				result: "Verification failed.",
				verifying: false,
			}));
			console.error(err);
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
			<Icon as={IconFolder} />
			{projectFolder ? "Change Folder" : "Select Project"}
		</Button>

		{projectFolder && (
			<Text fontSize="sm" color="gray.300" wordBreak="break-all">
			Selected: <Code colorScheme="whiteAlpha">{projectFolder}</Code>
			</Text>
		)}

		<Button
			onClick={onVerify}
			disabled={!projectFolder || verifying}
			loading={verifying}
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
