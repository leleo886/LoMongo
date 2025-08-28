<template>
  <v-card class="mx-auto">
    <!-- 应用图标和标题区域 -->
    <v-card-text class="text-center">
      <v-avatar class="about-avatar mb-4" size="64">
        <v-img 
          src="/icon.png"
          alt="Logo"
        ></v-img>
      </v-avatar>
      
      <h1 class="text-h4 font-bold mb-1">{{ appName }}</h1>
      <p class="text-subtitle-1 text-secondary mb-6">v {{ appVersion }}</p>
      
      <p class="text-body-1 mb-8 px-4">
        {{ appDescription }}
      </p>
    </v-card-text>
    
    <v-card-text class="pt-6">
		<v-list>
			<v-list-item
				rounded="xl"
			>
				<template v-slot:prepend>
					<v-icon icon="mdi-github"></v-icon>
				</template>
				<v-list-item-title>github</v-list-item-title>
				<template v-slot:append>
					<v-btn 
						icon="mdi-open-in-new" color="primary" variant="text"
						:href="githubRepo" target="_blank"
					></v-btn>
				</template>
			</v-list-item>
		</v-list>
    </v-card-text>
  </v-card>
</template>

<script>
import { getVersion } from '@tauri-apps/api/app';
export default {
	name: 'AboutPage',
	data() {
	return {
		// 应用信息 - 可根据实际情况修改
		appName: this.$t('app.name'),
		appVersion: '',
		appDescription: this.$t('app.description'),
		githubRepo: 'https://github.com/leleo886/LoMongo',
	}
	},
	mounted() {
		getVersion().then((version) => {
			this.appVersion = version;
		});
	}
}
</script>

