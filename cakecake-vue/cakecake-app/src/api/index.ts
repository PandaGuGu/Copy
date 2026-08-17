import { authApi } from './auth'
import { videoApi } from './video'
import { categoryApi } from './category'
import { bannerApi } from './banner'
import { dynamicApi, userApi, liveApi } from './user'
import { searchApi } from './search'
import { uploadApi } from './upload'
import { myApi } from './my'
import { spaceApi } from './space'
import { notificationApi } from './notification'
import { dmApi } from './dm'
import { favoriteApi } from './favorite'
import { historyApi } from './history'

export { authApi, videoApi, categoryApi, bannerApi, dynamicApi, userApi, liveApi, searchApi, uploadApi, myApi, spaceApi, notificationApi, dmApi, favoriteApi, historyApi }

export const api = {
  auth: authApi,
  video: videoApi,
  category: categoryApi,
  banner: bannerApi,
  dynamic: dynamicApi,
  user: userApi,
  live: liveApi,
  search: searchApi,
  upload: uploadApi,
  my: myApi,
  space: spaceApi,
  notification: notificationApi,
  dm: dmApi,
  favorite: favoriteApi,
  history: historyApi
}

export default api