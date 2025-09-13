package schema

public object AgentMethods {
  public const val authenticate: String = "authenticate"
  public const val initialize: String = "initialize"
  public const val session_cancel: String = "session/cancel"
  public const val session_load: String = "session/load"
  public const val session_new: String = "session/new"
  public const val session_prompt: String = "session/prompt"
}

public object ClientMethods {
  public const val fs_read_text_file: String = "fs/read_text_file"
  public const val fs_write_text_file: String = "fs/write_text_file"
  public const val session_request_permission: String = "session/request_permission"
  public const val session_update: String = "session/update"
  public const val terminal_create: String = "terminal/create"
  public const val terminal_kill: String = "terminal/kill"
  public const val terminal_output: String = "terminal/output"
  public const val terminal_release: String = "terminal/release"
  public const val terminal_wait_for_exit: String = "terminal/wait_for_exit"
}

public const val PROTOCOL_VERSION: Int = 1
