import { Exercise, ExerciseSession, LoadResult } from "@/models";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';

export async function openFolderDialog(): Promise<string | null> {
    const result = await open({
        directory: true,
    });

    return result;
}

export async function load(): Promise<LoadResult> {
    return invoke("load");
}

export async function getExercises(): Promise<Exercise[]> {
    return invoke("get_exercises");
}

export async function getLastExerciseSession(): Promise<ExerciseSession | undefined> {
    return invoke("get_last_exercise_session");
}

export async function verifyExercise(id: string): Promise<never> {
    return invoke("verify_exercise",  {
        id
    });
}

export async function getMarkdown(markdownName: string): Promise<string> {
    return invoke("get_markdown", {
        markdownName
    });
}