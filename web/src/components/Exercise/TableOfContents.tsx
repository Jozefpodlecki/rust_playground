import { Exercise } from "@/models";
import { Box } from "@chakra-ui/react";
import { useCallback } from "react";

interface Props {
    open: boolean;
    items: Exercise[];
    activeId: string;
    onSelect(exercise: Exercise): void;
}
  
const TableOfContents: React.FC<Props> = ({ open, items, activeId, onSelect }) => {

    const onClick = useCallback((event: React.MouseEvent<HTMLElement>) => {
        const id = event.currentTarget.dataset.id;
        const item = items.find(pr => pr.id === id);
        onSelect(item!);
    }, []);
 
    return (
    <div
        className={`w-64 bg-gray-800 p-6 overflow-y-auto transition-all duration-300 ${
            open ? "transform-none" : "-translate-x-full"
        }`}
        >
        <div>
            {items.map((item, index) => (
            <Box data-id={item.id} key={index}
                onClick={onClick}
                p="2"
                background={item.id === activeId ? "gray.800" : "gray.900" }
                _hover={{
                    bg: item.id === activeId ? "gray.600" : "gray.700",
                }}
                className="cursor-pointer">
                {item.name}
            </Box >
            ))}
        </div>
        </div>
    );
};

export default TableOfContents;
  