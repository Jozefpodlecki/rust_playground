import { useState } from "react";
import { IconFolder, IconCheck } from "@tabler/icons-react";
import { setFolder, verifyExercise } from "../../api";
import { useExercises } from "../../providers/ExerciseProvider";

interface State {
	folder: string;
	verifying: boolean;
	result: string;
}

const InputPanel: React.FC = () => {
	const [{
		folder,
		verifying,
		result
	}, setState] = useState<State>({
		folder: "",
		verifying: false,
		result: ""
	});
	const exercise = useExercises();
	const selectFolder = async () => {
	try {
		const folder = await setFolder();
		setState(state => {
			return {
				...state,
				folder
			}
		});
	} catch (err) {
		console.error("Folder selection failed", err);
	}
	};

	const verify = async () => {
		if (!folder) {
			return;
		}

		try {
			setState((prevState) => ({
			  ...prevState,
			  verifying: true,
			}));
			const response = await verifyExercise(exercise.current!.id);
			setState((prevState) => ({
			  ...prevState,
			  result: response,
			  verifying: false,
			}));
		  } catch (err) {
			setState((prevState) => ({
			  ...prevState,
			  result: "Verification failed.",
			  verifying: false,
			}));
			console.error(err);
		}
	};

  return (
    <div className="bg-gray-800 text-white p-6">
      <div className="flex flex-col gap-4">
        <button
          onClick={selectFolder}
          className="flex items-center bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded"
        >
          <IconFolder size={20} className="mr-2" />
          {folder ? "Change Folder" : "Browse..."}
        </button>

        {folder && (
          <p className="text-sm text-gray-300 break-all">
            Selected: <span className="text-white">{folder}</span>
          </p>
        )}

        <button
          onClick={verify}
          disabled={!folder || verifying}
          className="flex items-center bg-green-600 hover:bg-green-700 px-4 py-2 rounded"
        >
          <IconCheck size={20} className="mr-2" />
          {verifying ? "Verifying..." : "Verify"}
        </button>

        {result && <p className="text-sm text-yellow-400">{result}</p>}
      </div>
    </div>
  );
};

export default InputPanel;
