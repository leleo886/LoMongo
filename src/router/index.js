import { createRouter, createWebHistory } from 'vue-router'
import Collections from '../components/collections.vue'
import Coll from '../components/coll.vue'
import Server from '../components/server.vue'
import About from '../components/about.vue'
import QueryExample from '../components/query.vue'

const routes = [
	{
		path: '/',
		name: 'Home',
		component: Server,
	},
	{
		path: '/example',
		name: 'QueryExample',
		component: QueryExample,
	},
	{
		path: '/about',
		name: 'About',
		component: About,
	},
	{
		path: '/collections',
		name: 'Collections',
		component: Collections,
	},
	{
		path: '/coll',
		name: 'Coll',
		component: Coll,
	},
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})
export default router