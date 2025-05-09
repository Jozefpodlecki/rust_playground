import { Exercise } from "@/models";
import { useCallback } from "react";

interface Props {
    open: boolean;
    items: Exercise[];
    onSelect(exercise: Exercise): void;
}
  
const TableOfContents: React.FC<Props> = ({ open, items, onSelect }) => {

    const onClick = useCallback((event: React.MouseEvent<HTMLElement>) => {
        const id = Number(event.currentTarget.dataset.id);
        const item = items.find(pr => pr.id === id);
        onSelect(item!);
    }, []);
 
    return (
    <div
        className={`w-64 bg-gray-800 p-6 overflow-y-auto transition-all duration-300 ${
            open ? "transform-none" : "-translate-x-full"
        }`}
        >
        <ul>
            {items.map((item, index) => (
            <li data-id={item.id} key={index}
                onClick={onClick}
                className="text-blue-500 hover:underline">
                {item.name}
            </li>
            ))}
        </ul>
        </div>
    );
};

export default TableOfContents;
  