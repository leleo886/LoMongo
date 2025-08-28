import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { createI18n } from 'vue-i18n'
import messages from './messages'

// Vuetify
import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import '@mdi/font/css/materialdesignicons.css'
import colors from 'vuetify/lib/util/colors'

const i18n = createI18n({
	legacy: false,
	locale: 'zh',
	messages
})

const vuetify = createVuetify({
	components,
	directives,
	theme: {
		themes: {
			light: {
				colors: {
					primary: colors.green.darken3, 
					secondary: colors.lightGreen.darken1, 
					error: colors.red.darken1, 
					warning: colors.amber.darken1,
					info: colors.lightBlue.darken2
				}
				
			},
		},
	},
})


const app = createApp(App);
app.config.globalProperties.$globalConnectedStates = [],
app.config.warnHandler = () => { };
app.use(vuetify).use(router).use(i18n).mount('#app')

