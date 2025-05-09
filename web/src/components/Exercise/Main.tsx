import { useState, useEffect } from "react";
import TableOfContents from "./TableOfContents";
import MarkdownViewer from "./MarkdownViewer";
import InputPanel from "./InputPanel";
import { IconList } from "@tabler/icons-react";
import { useExercises } from "@/providers/ExerciseProvider";
import { getFakeExercises, getMarkdown } from "@/api";
import TopBar from "../TopBar";
import { Exercise } from "@/models";

const Main: React.FC = () => {
	const [isDrawerOpen, setIsDrawerOpen] = useState(true);
	const [markdown, setMarkdown] = useState<string>("");
	const [tableOfContents, setTableOfContents] = useState<Exercise[]>([]);
	const {
		current
	} = useExercises();

	useEffect(() => {
		onLoad();
	}, []);

	async function onLoad() {

		if(current) {
			const markdown = await getMarkdown(current!.id);
			setMarkdown(markdown);
		}

	

		const exercises = await getFakeExercises();
		setTableOfContents(exercises);
	}

	const onSelect = (exercise: Exercise) => {

	}

	return (
	<section className="flex flex-col h-full w-full bg-gray-900 text-white">
		<TopBar/>
		<div className="flex h-full w-full p-6 gap-1">
			<TableOfContents open={isDrawerOpen} items={tableOfContents} onSelect={onSelect} />
			<MarkdownViewer markdown={markdown} />
			<InputPanel />
		</div>
	</section>
	);
};

export default Main;
