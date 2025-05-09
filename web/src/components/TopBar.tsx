import { useExercises } from "../providers/ExerciseProvider";
import { useApp } from "../providers/AppProvider";
import { IconBrandGithub, IconHome } from "@tabler/icons-react";
import { Link, useLocation } from "react-router-dom";

const TopBar: React.FC = () => {
    const { progressPercent } = useExercises();
    const { rustVersion, version, githubLink } = useApp();
    const location = useLocation();
    const isLandingPage = location.pathname === "/";

    return <nav className="bg-gray-900 text-white h-8 flex items-center px-6 shadow-sm justify-between w-full">
        <div className="flex-1 h-full bg-gray-700 relative">
            <div
                className="h-full bg-blue-500 transition-all duration-300"
                style={{ width: `${progressPercent}%` }}
            />
            <div className="absolute inset-0 flex items-center justify-center text-xs font-medium text-white">
                {progressPercent}% Complete
            </div>
        </div>
        <div className="flex flex-row m-40">
            {isLandingPage ? null : <Link to="/" className="w-10 flex justify-center items-center">
                <IconHome size={20} />
            </Link>}
            <a href={githubLink} className="w-10 flex justify-center items-center">
                <IconBrandGithub size={20} />
            </a>
            <div className="m-4 text-md text-gray-300">
                {rustVersion} • v{version}
            </div>
        </div>
    </nav>
}

export default TopBar;