import { Exercise, LoadResult } from "@/models";
import { invoke } from "@tauri-apps/api/core";

export async function load(): Promise<LoadResult> {
    return invoke("load");
}

export async function getExercises(): Promise<Exercise[]> {
    return invoke("get_exercises");
}

export async function setFolder(): Promise<string> {
    return invoke("set_folder");
}

export async function verifyExercise(id: number): Promise<never> {
    return invoke("verify_exercise",  {
        id
    });
}

export async function getMarkdown(exerciseId: number): Promise<string> {
    return invoke("get_markdown", {
        exerciseId
    });
}