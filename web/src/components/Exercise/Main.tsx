import { useState, useEffect } from "react";
import TableOfContents from "./TableOfContents";
import MarkdownViewer from "./MarkdownViewer";
import InputPanel from "./InputPanel";
import { IconList } from "@tabler/icons-react";
import { useExercises } from "@/providers/ExerciseProvider";
import { getExercises, getMarkdown } from "@/api";
import TopBar from "../TopBar";
import { Exercise } from "@/models";

interface State {
	markdown: string;
	activeExerciseId: string;
}

const Main: React.FC = () => {
	const [isDrawerOpen, setIsDrawerOpen] = useState(true);
	const [{
		markdown,
		activeExerciseId
	}, setState] = useState<State>({
		markdown: "",
		activeExerciseId: ""
	});
	
	const {
		currentExercise,
		currentSession,
		exercises
	} = useExercises();

	useEffect(() => {
		onLoad();
	}, []);

	async function onLoad() {

		try {

		const markdown = await getMarkdown(currentExercise.markdown);

		setState(pr => {
			return {
				...pr,
				markdown,
				activeExerciseId: currentExercise.id
			}
		});

		} catch (error) {
			console.log(error);
		}
	

	}

	const onSelect = async (exercise: Exercise) => {
		const markdown = await getMarkdown(exercise.markdown);
			setState(pr => {
			return {
				...pr,
				markdown,
				activeExerciseId: exercise.id
			}
		});
	}

	return (
	<section className="flex flex-col h-full w-full bg-gray-900 text-white">
		<TopBar/>
		<div className="flex h-full w-full p-6 gap-1">
			<TableOfContents
				open={isDrawerOpen}
				items={exercises}
				activeId={activeExerciseId}
				onSelect={onSelect} />
			<MarkdownViewer markdown={markdown} />
			<InputPanel />
		</div>
	</section>
	);
};

export default Main;
