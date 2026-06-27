// Package service provides business-logic service layers that sit between
// HTTP handlers and GORM data access.  Handlers should delegate CRUD and
// domain rules to services instead of embedding SQL directly.
package service

import "gorm.io/gorm"

// Services holds all service instances (initialised once, wired via DI).
type Services struct {
	Video   *VideoService
	User    *UserService
	Comment *CommentService
}

// NewServices constructs the service bundle.
func NewServices(db *gorm.DB, cfg interface{}) *Services {
	return &Services{
		Video:   &VideoService{DB: db},
		User:    &UserService{DB: db},
		Comment: &CommentService{DB: db},
	}
}
