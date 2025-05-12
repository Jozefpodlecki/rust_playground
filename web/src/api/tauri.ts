import { CreateExerciseSession, Exercise, ExerciseSession, LoadResult, UpdateExerciseSession } from "@/models";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';

export async function openFolderDialog(): Promise<string | null> {
    const result = await open({
        directory: true,
    });

    return result;
}

export async function createSession(payload: CreateExerciseSession): Promise<ExerciseSession> {
    const id = await invoke<ExerciseSession>("create_session",  {
        payload
    });

    return id;
}

export async function updateSession(payload: UpdateExerciseSession): Promise<ExerciseSession> {
    return invoke("update_session",  {
        payload
    });
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

export async function verifyExercise(sessionId: string): Promise<never> {
    return invoke("verify_exercise",  {
        sessionId
    });
}

export async function getMarkdown(markdownName: string): Promise<string> {
    return invoke("get_markdown", {
        markdownName
    });
}