package service

import (
	"gorm.io/gorm"

	"minibili/internal/model"
)

// VideoService encapsulates video-domain business rules.
type VideoService struct {
	DB *gorm.DB
}

// ListPublished returns published videos with optional pagination.
func (s *VideoService) ListPublished(page, pageSize int) ([]model.Video, int64, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	var total int64
	q := s.DB.Model(&model.Video{}).Where("status = ?", "published")
	if err := q.Count(&total).Error; err != nil {
		return nil, 0, err
	}
	var list []model.Video
	offset := (page - 1) * pageSize
	if err := q.Order("created_at DESC").Offset(offset).Limit(pageSize).Find(&list).Error; err != nil {
		return nil, 0, err
	}
	return list, total, nil
}

// ListByUser returns videos owned by uid.
func (s *VideoService) ListByUser(uid uint64, page, pageSize int) ([]model.Video, int64, error) {
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	var total int64
	q := s.DB.Model(&model.Video{}).Where("user_id = ?", uid)
	if err := q.Count(&total).Error; err != nil {
		return nil, 0, err
	}
	var list []model.Video
	offset := (page - 1) * pageSize
	if err := q.Order("created_at DESC").Offset(offset).Limit(pageSize).Find(&list).Error; err != nil {
		return nil, 0, err
	}
	return list, total, nil
}

// Publish transitions a video from processing/draft to published.
func (s *VideoService) Publish(id uint64) error {
	return s.DB.Model(&model.Video{}).Where("id = ?", id).Updates(map[string]interface{}{
		"status": "published",
	}).Error
}

// Reject marks a video as rejected.
func (s *VideoService) Reject(id uint64) error {
	return s.DB.Model(&model.Video{}).Where("id = ?", id).Updates(map[string]interface{}{
		"status": "rejected",
	}).Error
}

// Delete soft-deletes a video by id.
func (s *VideoService) Delete(id uint64) error {
	return s.DB.Delete(&model.Video{}, id).Error
}

// GetByID fetches a single video.
func (s *VideoService) GetByID(id uint64) (*model.Video, error) {
	var v model.Video
	if err := s.DB.First(&v, id).Error; err != nil {
		return nil, err
	}
	return &v, nil
}
