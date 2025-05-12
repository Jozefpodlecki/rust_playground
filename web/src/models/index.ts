
export interface LoadResult {
    appName: string,
    rustVersion: string,
    githubLink: string,
    version: string,
	loadedOn: string;
}

export interface CreateExerciseSession {
    exerciseId: string;
    commandArgs: string;
    folderPath: string;
}


export interface UpdateExerciseSession {
    id: string;
    folderPath: string | null;
    commandArgs: string | null;
    completedOn: string | null;
}


export interface Exercise {
    id: string;
    name: string;
    markdown: string;
}

export interface ExerciseSession {
    id: string;
    exerciseId: string;
    folderPath: string,
    startedOn: string;
    completedOn?: string;
}