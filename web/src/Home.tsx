import { Link } from "react-router-dom";
import { IconDualScreen, IconFlask2 } from "@tabler/icons-react";
import TopBar from "./components/TopBar";

const Home: React.FC = () => {

    return (
        <section className="h-full w-full flex flex-col">
            <TopBar/>
            <div className="flex flex-1 gap-8 p-8 justify-center items-center bg-gray-950 text-white">
                <Link
                    to="/exercise"
                    className="flex flex-col items-center hover:text-blue-400 cursor-pointer transition"
                >
                    <IconFlask2 size={96} />
                    <span className="mt-2 text-lg font-medium">Exercises</span>
                </Link>

                <Link
                    to="/examples"
                    className="flex flex-col items-center hover:text-green-400 cursor-pointer transition"
                >
                    <IconDualScreen size={96} />
                    <span className="mt-2 text-lg font-medium">Examples</span>
                </Link>
            </div>
        </section>
    );
};

export default Home;
