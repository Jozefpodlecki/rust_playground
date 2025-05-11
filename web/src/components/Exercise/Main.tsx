import { useState, useEffect } from "react";
import TableOfContents from "./TableOfContents";
import MarkdownViewer from "./MarkdownViewer";
import InputPanel from "./InputPanel";
import { IconList } from "@tabler/icons-react";
import { useExercises } from "@/providers/ExerciseProvider";
import { getExercises, getMarkdown } from "@/api";
import TopBar from "../TopBar";
import { Exercise } from "@/models";

const Main: React.FC = () => {
	const [isDrawerOpen, setIsDrawerOpen] = useState(true);
	const [markdown, setMarkdown] = useState<string>("");
	const {
		current,
		exercises
	} = useExercises();

	useEffect(() => {
		onLoad();
	}, []);

	async function onLoad() {

		try {

		let exerciseId = exercises[0].id;
			
		if(current) {
			exerciseId = current.exerciseId;
		}

		const exercise = exercises.find(pr => pr.id === exerciseId)!;

		const markdown = await getMarkdown(exercise.markdown);
		setMarkdown(markdown);

		} catch (error) {
			console.log(error);
		}
	

	}

	const onSelect = async (exercise: Exercise) => {
		const markdown = await getMarkdown(exercise.markdown);
		setMarkdown(markdown);
	}

	return (
	<section className="flex flex-col h-full w-full bg-gray-900 text-white">
		<TopBar/>
		<div className="flex h-full w-full p-6 gap-1">
			<TableOfContents
				open={isDrawerOpen}
				items={exercises}
				activeId={""}
				onSelect={onSelect} />
			<MarkdownViewer markdown={markdown} />
			<InputPanel projectFolder={current?.folderPath || null} />
		</div>
	</section>
	);
};

export default Main;
