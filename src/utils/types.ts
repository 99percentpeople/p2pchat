export type Action = {
  name: string;
  icon: string;
  action: () => void;
};

export type FileInfo = {
  name: string;
  size: number;
  createTime?: Date;
  modifyTime?: Date;
};
export type FileSource = {
  local?: string;
  remote?: string;
};
export type Setting = {
  recvPath: string;
};

export type GroupId = string;
export type PeerId = string;
export type GroupMessage = {
  message: Message;
  timestamp: number;
  source?: string;
};

export type UserInfo = {
  name: string;
  avatar: string;
  status: "online" | "offline";
};

export type GroupState = {
  subscribers: PeerId[];
  history: GroupMessage[];
};

export type Message = {
  text?: string;
  file?: FileInfo;
};

export type GroupInfo = {
  name: string;
  description: string | null;
};
export type Multiaddr = string;
