<template>
  <div class="live-list-page">
    <div class="live-list-header">
      <h2 class="live-list-title">直播</h2>
      <div class="live-list-actions">
        <el-button v-if="isLoggedIn" type="primary" size="default" @click="$router.push('/minibili/live/create')">
          我要开播
        </el-button>
      </div>
    </div>

    <div v-loading="loading" class="live-list-body">
      <div v-if="featured.length" class="live-featured">
        <LiveRoomCard class="lf-big" :room="featured[0]" @click="goRoom(featured[0].room_id)" />
        <LiveRoomCard
          v-for="(room, i) in featured.slice(1, 6)"
          :key="room.room_id"
          :class="['lf-small', 'lf-small--' + i]"
          :room="room"
          @click="goRoom(room.room_id)"
        />
      </div>

      <div v-if="!loading && rooms.length === 0 && featured.length === 0" class="live-list-empty">
        <p class="live-empty-icon">📡</p>
        <h3>暂无直播</h3>
        <p>当前没有主播在线，稍后再来看看</p>
      </div>

      <div v-else class="live-grid">
        <LiveRoomCard
          v-for="room in rooms"
          :key="room.id"
          :room="room"
          @click="goRoom(room.id)"
        />
      </div>

      <div v-if="total > rooms.length" class="live-list-more">
        <el-button :loading="loadingMore" @click="loadMore">加载更多</el-button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { listLiveRooms, getFeaturedLiveRooms } from "@/api/live";
import { getAccessToken } from "@/utils/authTokens";
import LiveRoomCard from "@/components/live/LiveRoomCard.vue";

export default {
  name: "LiveRoomList",
  components: { LiveRoomCard },
  setup() {
    const router = useRouter();
    const loading = ref(true);
    const loadingMore = ref(false);
    const rooms = ref([]);
    const featured = ref([]);
    const page = ref(1);
    const total = ref(0);
    const pageSize = 20;

    const isLoggedIn = ref(!!getAccessToken());

    async function loadFeatured() {
      try {
        const res = await getFeaturedLiveRooms();
        const arr = (res && res.data) || res || [];
        featured.value = (Array.isArray(arr) ? arr : []).map((r) => ({
          room_id: r.room_id,
          id: r.room_id,
          title: r.title,
          cover_url: r.cover_url,
          avatar_url: r.avatar_url || "",
          viewer_count: r.viewer_count || 0,
          status: r.status || "live"
        }));
      } catch {
        featured.value = [];
      }
    }

    async function fetchRooms(reset = false) {
      if (reset) page.value = 1;
      try {
        const res = await listLiveRooms({ status: "live", page: page.value, page_size: pageSize });
        const data = res.data || res;
        const list = data.data ? (data.data.rooms || data.data.list || []) : (data.list || data.rooms || []);
        total.value = data.total || 0;
        if (reset) {
          rooms.value = list;
        } else {
          rooms.value = rooms.value.concat(list);
        }
      } catch (e) {
        ElMessage.warning("加载直播列表失败");
      } finally {
        loading.value = false;
        loadingMore.value = false;
      }
    }

    function loadMore() {
      loadingMore.value = true;
      page.value++;
      fetchRooms(false);
    }

    function goRoom(roomId) {
      router.push(`/minibili/live/${roomId}`);
    }

    onMounted(() => { loadFeatured(); fetchRooms(true); });

    return { loading, loadingMore, rooms, featured, total, isLoggedIn, loadMore, goRoom };
  }
};
</script>

<style scoped>
.live-list-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px 24px;
}
.live-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}
.live-list-title {
  font-size: 22px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
}
.live-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}
.live-list-empty {
  text-align: center;
  padding: 80px 0;
  color: var(--color-text-secondary);
}
.live-empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}
.live-list-more {
  text-align: center;
  margin-top: 24px;
}

/* 直播运营置顶：1 大 + 5 小 bento */
.live-featured {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: repeat(3, auto);
  gap: 12px;
  margin-bottom: 20px;
}
.lf-big {
  grid-column: 1 / span 2;
  grid-row: 1 / span 2;
}
.lf-small {
  min-height: 0;
}
.lf-small--0 { grid-column: 3; grid-row: 1; }
.lf-small--1 { grid-column: 3; grid-row: 2; }
.lf-small--2 { grid-column: 3; grid-row: 3; }
.lf-small--3 { grid-column: 1; grid-row: 3; }
.lf-small--4 { grid-column: 2; grid-row: 3; }
</style>
