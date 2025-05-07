interface Props {
    open: boolean;
    items: string[];
}
  
const TableOfContents: React.FC<Props> = ({ open, items }) => {
    return (
    <div
        className={`fixed inset-y-0 left-0 w-64 bg-gray-800 p-6 overflow-y-auto transition-all duration-300 ${
            open ? "transform-none" : "-translate-x-full"
        }`}
        >
        <h3 className="text-xl font-semibold mb-4">Table of Contents</h3>
        <ul>
            {items.map((item, index) => (
            <li key={index} className="text-blue-500 hover:underline">
                <a href={`#${item.replace(/\s+/g, "-").toLowerCase()}`}>{item}</a>
            </li>
            ))}
        </ul>
        </div>
    );
};

export default TableOfContents;
  