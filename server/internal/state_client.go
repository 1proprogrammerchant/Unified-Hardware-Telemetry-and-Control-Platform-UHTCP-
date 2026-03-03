package internal

// Client that reads state from core IPC

type StateClient struct{}

func NewStateClient() *StateClient { return &StateClient{} }

func (s *StateClient) GetState() ([]byte, error) { return []byte("{}"), nil }
