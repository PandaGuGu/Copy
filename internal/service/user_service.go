package service

import (
	"gorm.io/gorm"

	"minibili/internal/model"
)

// UserService encapsulates user-domain business rules.
type UserService struct {
	DB *gorm.DB
}

// List returns a paginated user list with optional keyword search.
func (s *UserService) List(page, pageSize int, statusFilter, q string) ([]model.User, int64, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	qb := s.DB.Model(&model.User{})
	if statusFilter != "" {
		qb = qb.Where("status = ?", statusFilter)
	}
	if q != "" {
		like := "%" + q + "%"
		qb = qb.Where("username LIKE ? OR cake_id LIKE ? OR nickname LIKE ?", like, like, like)
	}
	var total int64
	if err := qb.Count(&total).Error; err != nil {
		return nil, 0, err
	}
	var list []model.User
	offset := (page - 1) * pageSize
	if err := qb.Order("id DESC").Offset(offset).Limit(pageSize).Find(&list).Error; err != nil {
		return nil, 0, err
	}
	return list, total, nil
}

// GetByID returns a single user.
func (s *UserService) GetByID(id uint64) (*model.User, error) {
	var u model.User
	if err := s.DB.First(&u, id).Error; err != nil {
		return nil, err
	}
	return &u, nil
}

// Ban sets a user's status to "banned".
func (s *UserService) Ban(id uint64) error {
	return s.DB.Model(&model.User{}).Where("id = ?", id).Update("status", "banned").Error
}

// Unban sets a user's status back to "active".
func (s *UserService) Unban(id uint64) error {
	return s.DB.Model(&model.User{}).Where("id = ?", id).Update("status", "active").Error
}

// Delete marks a user as disabled/deleted.
func (s *UserService) Delete(id uint64) error {
	return s.DB.Model(&model.User{}).Where("id = ?", id).Update("status", "disabled").Error
}
