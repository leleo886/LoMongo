<template>
    <v-app>
		<v-system-bar color="primary"></v-system-bar>
		<v-app-bar color="primary">
			<template v-slot:prepend>
				<v-btn icon="mdi-home" :to="{ name: 'Home' }"></v-btn>
			</template>
			<v-app-bar-title >{{ appName }}</v-app-bar-title>
	
			<template v-slot:append>
			<v-menu>
				<template v-slot:activator="{ props }">
					<v-btn v-bind="props" icon="mdi-dots-vertical"></v-btn>
				</template>
				<v-list>
					<v-list-item
						density="comfortable"
						style="text-align: center;"
						@click="dialog = true"
					>	
						<v-icon color="secondary" icon="mdi-translate"></v-icon>
					</v-list-item>
					<v-list-item
						density="comfortable"
						style="text-align: center;"
						@click="this.$router.push('/example')"
					>	
						<v-icon color="secondary" icon="mdi-database-search"></v-icon>
					</v-list-item>
					<v-list-item
						density="comfortable"
						style="text-align: center;"
						@click="this.$router.push('/about')"
					>	
						<v-icon color="secondary" icon="mdi-information-outline"></v-icon>
					</v-list-item>
				</v-list>
			</v-menu>
			</template>
      	</v-app-bar>

		<v-main>
			<router-view></router-view>
		</v-main>
		
		<v-snackbar
		v-model="snackbar"
		:timeout="snackbarTimeout"
		:color="snackbarColor"
		>
			{{ snackbarText }}
		</v-snackbar>

		<v-dialog width="auto" scrollable v-model="dialog">

		<template v-slot:default="{ isActive }">
			<v-card>
				<v-card-text class="px-4 mr-4" style="height: 150px;">
					<v-radio-group v-model="lang">
						<v-radio
							v-for="item in langList"
							true-icon="mdi-check"
							false-icon=""
							:label="item.label"
							:value="item.value"
							density="comfortable"
							color="secondary"
						></v-radio>
					</v-radio-group>
				</v-card-text>

				<v-divider></v-divider>

				<v-card-actions>
					<v-btn
						color="primary"
						text="OK"
						variant="text"
						@click="saveLang(isActive)"
					></v-btn>
				</v-card-actions>
			</v-card>
		</template>
		</v-dialog>
    </v-app>
</template>

<script>
import { getName } from '@tauri-apps/api/app';
import { locale, type } from '@tauri-apps/plugin-os';
import { load } from '@tauri-apps/plugin-store';

export default {
	name: 'App',
	computed: {
		routeClass() {
			return this.$route.name;
		},
	},
	data() {
		return {
			appName:'',
			snackbar: false,
			snackbarText: '',
			snackbarColor: '',
			snackbarTimeout: 1000,
			dialog: false,
			lang: '',
			langList: [
				{label: '简体中文', value: 'zh-CN'},
				{label: 'English', value: 'en-US'},
			],
		};
	},
	provide() {
    	return {
			showSnackbar: this.showSnackbar,
		};
	},

	async mounted() {
		getName().then((name) => {
			this.appName = name;
		});
		const store = await load('store.json', { autoSave: false });
		const val = await store.get('lang');
		if (val) {
			this.lang = val;
			this.$i18n.locale = val;
		}else{
			locale().then((lang) => {
				this.lang = lang;
				this.$i18n.locale = lang;
			})
		}
	},
	methods: {
		showSnackbar( snackbarText, snackbarColor, snackbarTimeout) {
			this.snackbarText = snackbarText;
			this.snackbarColor = snackbarColor;
			this.snackbarTimeout = snackbarTimeout;
			this.snackbar = true;
		},
		async saveLang(isActive){
			this.$i18n.locale = this.lang;
			const store = await load('store.json', { autoSave: false });
			await store.set('lang', this.lang);
			await store.save();
			isActive.value = false;
		}
	}
}
</script>