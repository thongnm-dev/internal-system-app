export type DailyWorkNoteResult = {
  id: number;
  username: string;
  content: string;
  note_date: string;
  status: string;
  created_at: string;
};

export type CreateDailyNoteRequest = {
  content: string;
  note_date: string;
  status: string;
};

export type DailyNoteDateCountResult = {
  note_date: string;
  note_count: number;
};
