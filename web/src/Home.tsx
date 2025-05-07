import { Link } from "react-router-dom";
import { IconDualScreen, IconFlask2 } from "@tabler/icons-react";
import { useExercises } from "./ExerciseProvider";
import { useApp } from "./appProvider";

const Home: React.FC = () => {
    const { progressPercent } = useExercises();
    const { rustVersion, version } = useApp();

    return (
        <section className="h-full w-full flex flex-col">
            <div className="bg-gray-900 text-white h-6 flex items-center px-6 shadow-sm justify-between w-full">
                <div className="w-7/10 h-full bg-gray-700 relative">
                    <div
                        className="h-full bg-blue-500 transition-all duration-300"
                        style={{ width: `${progressPercent}%` }}
                    />
                    <div className="absolute inset-0 flex items-center justify-center text-xs font-medium text-white">
                        {progressPercent}% Complete
                    </div>
                </div>
                <div className="text-sm text-gray-300">
                    {rustVersion} • v{version}
                </div>
            </div>
            <div className="flex flex-1 gap-8 p-8 justify-center items-center bg-gray-950 text-white">
                <Link
                    to="/exercise"
                    className="flex flex-col items-center hover:text-blue-400 cursor-pointer transition"
                >
                    <IconFlask2 size={48} />
                    <span className="mt-2 text-lg font-medium">Exercises</span>
                </Link>

                <Link
                    to="/examples"
                    className="flex flex-col items-center hover:text-green-400 cursor-pointer transition"
                >
                    <IconDualScreen size={48} />
                    <span className="mt-2 text-lg font-medium">Examples</span>
                </Link>
            </div>
        </section>
    );
};

export default Home;
