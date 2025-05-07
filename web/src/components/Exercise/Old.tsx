// import { useEffect, useState } from "react";
// import Markdown from "react-markdown";
// import remarkGfm from "remark-gfm";
// import TableOfContents from "./TableOfContents";
// import InputPanel from "./InputPanel";

// const mockMarkdown = `
// # Background Worker

// This is a test markdown.

// ## Introduction

// Some content...

// ## How It Works

// More content...
// `;

// const Exercise: React.FC = () => {
//   const [markdown, setMarkdown] = useState("");
//   const [tocOpen, setTocOpen] = useState(true);

//   useEffect(() => {
//     // Simulate fetch
//     setMarkdown(mockMarkdown);
//   }, []);

//   return (
//     <section className="flex h-full w-full">
//       {tocOpen && (
//         <div className="w-64 bg-gray-900 text-white p-4 border-r border-gray-700">
//           <TableOfContents markdown={markdown} onClose={() => setTocOpen(false)} />
//         </div>
//       )}

//       <main className="flex-1 flex flex-col gap-4 p-6 bg-gray-950 text-white overflow-y-auto">
//         <div className="prose prose-invert max-w-full">
//           <Markdown remarkPlugins={[remarkGfm]}>{markdown}</Markdown>
//         </div>
//         <InputPanel />
//       </main>
//     </section>
//   );
// };

// export default Exercise;



// // import { useEffect, useState } from "react";
// // import Markdown from "react-markdown";
// // import rehypeHighlight from "rehype-highlight";
// // import { IconList, IconFolder } from "@tabler/icons-react";
// // import { getMarkdown } from "../../api";

// // const Exercise: React.FC = () => {
// //   const [markdown, setMarkdown] = useState<string>("");
// //   const [loading, setLoading] = useState<boolean>(true);
// //   const [tableOfContents, setTableOfContents] = useState<string[]>([]);
// //   const [isDrawerOpen, setIsDrawerOpen] = useState<boolean>(false);

// //   useEffect(() => {
// //     const hardcodedToc = [
// //       "Introduction",
// //       "Setup",
// //       "Writing Code",
// //       "Testing",
// //       "Conclusion",
// //     ];

// //     const hardcodedMarkdown = `
// // # Introduction
// // This is the introduction to the exercise.

// // ## Setup
// // Follow these steps to set up the environment.

// // ### Writing Code
// // This is where you write the code.

// // #### Testing
// // Test your solution here.

// // ##### Conclusion
// // Here we conclude the exercise.
// // `;

// //     setMarkdown(hardcodedMarkdown);
// //     setTableOfContents(hardcodedToc);
// //     setLoading(false);
// //   }, []);

// //   return (
// //     <section className="flex h-full w-full bg-gray-900 text-white">
// //       <div className="flex flex-col w-full">
// //         <div className="bg-gray-800 h-12 flex items-center justify-between px-6">
// //           <button
// //             className="text-white flex items-center"
// //             onClick={() => setIsDrawerOpen(!isDrawerOpen)}
// //           >
// //             <IconList size={24} />
// //             <span className="ml-2">Table of Contents</span>
// //           </button>
// //           <div className="text-sm text-gray-300 flex items-center gap-2">
// //             <span>v1.57.0</span>
// //             <span>•</span>
// //             <span>v0.2.0</span>
// //           </div>
// //         </div>

// //         <div className="flex h-full w-full p-6 gap-6">
// //           <div
// //             className={`fixed inset-y-0 left-0 w-64 bg-gray-800 p-6 overflow-y-auto transition-all duration-300 ${
// //               isDrawerOpen ? "transform-none" : "transform -translate-x-full"
// //             }`}
// //           >
// //             <h3 className="text-xl font-semibold mb-4">Table of Contents</h3>
// //             <ul>
// //               {tableOfContents.length > 0 ? (
// //                 tableOfContents.map((item, index) => (
// //                   <li key={index} className="text-blue-500 hover:underline">
// //                     <a href={`#${item.replace(/\s+/g, "-").toLowerCase()}`}>{item}</a>
// //                   </li>
// //                 ))
// //               ) : (
// //                 <p>No contents available</p>
// //               )}
// //             </ul>
// //           </div>

// //           <div className="w-2/3 bg-gray-800 rounded-xl shadow p-6 overflow-y-auto">
// //             {loading && <p className="text-gray-500">Loading exercise...</p>}
// //             {!loading && (
// //               <Markdown rehypePlugins={[rehypeHighlight]}>{markdown}</Markdown>
// //             )}
// //           </div>

// //           <div className="w-1/3 bg-gray-800 rounded-xl shadow p-6">
// //             <h2 className="text-xl font-semibold mb-4">Select Folder</h2>
// //             <button
// //               className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
// //               onClick={() => {
// //                 console.log("Pick folder...");
// //               }}
// //             >
// //               <IconFolder size={20} className="mr-2" />
// //               Browse...
// //             </button>
// //           </div>
// //         </div>
// //       </div>
// //     </section>
// //   );
// // };

// // export default Exercise;
