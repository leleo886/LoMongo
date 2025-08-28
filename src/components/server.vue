<template>
  <div class="py-3">

    <div class="header-container d-flex justify-space-between align-center mb-4 px-2">
      <span class="text-h5 font-weight-bold">MongoDB Servers</span>
      <v-btn 
        color="secondary" 
        @click="resetForm();editing = false;dialog = true"
        elevation="2"
        size="large"
        class="add-btn"
      >
        <v-icon>mdi-plus</v-icon>
      </v-btn>
    </div>

    <div v-if="servers.length > 0" class="px-2 mb-4">
      <div class="d-flex flex-column ga-3">
        <v-card 
          v-for="(server, index) in servers" 
          :key="index"
          elevation="3" 
        >
          <v-card-text class="px-4 pb-2 d-flex justify-space-between">
            <v-list dense>
              <v-list-item class="py-1">
                <v-list-item-icon class="mr-3">
                  <v-icon color="primary">mdi-server</v-icon>
                </v-list-item-icon>
                <v-list-item-content>
                  <v-list-item-title >
                    {{ server.type === 'ssh' ? server.sshHost : server.mongoHost }} : {{ server.type === 'ssh' ? server.sshPort : server.mongoPort }}
                  </v-list-item-title>
                </v-list-item-content>
              </v-list-item>
              
              <v-list-item class="py-1">
                <v-list-item-icon class="mr-3">
                  <v-icon color="primary">mdi-database</v-icon>
                </v-list-item-icon>
                <v-list-item-content>
                  <v-list-item-title class="text-body-1">
                    {{ $t("server.database") }}: {{ server.dbName }}
                  </v-list-item-title>
                </v-list-item-content>
              </v-list-item>
              
              <v-list-item class="py-1">
                <v-list-item-icon class="mr-3">
                  <v-icon color="primary">mdi-account</v-icon>
                </v-list-item-icon>
                <v-list-item-content>
                  <v-list-item-title class="text-body-1">
					{{ $t("server.sshAuthentication") }}: 
					<span :style="{color: server.type==='ssh' ? 'blue' : ''}">
						{{ server.type==='ssh' ? $t("server.enabled") : $t("server.notEnabled") }}
					</span>
					<br>
                    {{ $t("server.authentication") }}:
					<span :style="{color: server.authMethod==='none' ? '' : 'blue'}">
						{{ server.authMethod==="none" ? $t("server.notEnabled") : $t("server.enabled") }}
					</span> 
                  </v-list-item-title>
                </v-list-item-content>
              </v-list-item>
            </v-list>
			<v-btn 
				icon="mdi-pencil-outline" 
				color="secondary" 
				size="small" 
				@click="editServer(index)">
			</v-btn>
          </v-card-text>
          
          <v-card-actions class="d-flex justify-space-between">
			<v-btn 
                icon 
                small 
                color="error"
                class="mx-2"
                @click="removeServerCheck(index)"
              >
                <v-icon>mdi-trash-can</v-icon>
            </v-btn>
			<div>
				<v-btn
					color="error"
					:disabled="!connectedStates[index]"
					@click="disConnection(index)"
				>
					<v-icon>mdi-link-variant-off</v-icon>
					{{ $t("server.disconnect") }}
				</v-btn>
				<v-btn
					:color="connectedStates[index] ? 'secondary' : ''"
					class="mr-2"
					@click="testConnection(index)"
					:loading="loadingStates[index]"
				>
					<v-icon>mdi-link-variant</v-icon>
					{{ $t("server.connect") }}
				</v-btn>
			</div>
            
          </v-card-actions>
        </v-card>
      </div>
    </div>

    <div v-else>
      <v-card class="text-center elevation-1">
        <v-card-text class="py-10 px-4">
          <v-icon size="64" class="text-grey-400 mb-4" color="warning">mdi-server-off</v-icon>
          <p class="text-h6 text-grey-500">{{ $t("server.serverNotAdded") }}</p>
          <p class="text-body-1 text-grey-400 mt-3">{{ $t("server.addServerMsg") }}</p>
        </v-card-text>
      </v-card>
    </div>

    <!-- 表单对话框  -->
    <v-dialog
		v-model="dialog"
		hide-overlay
    >
      <v-card>
        <v-toolbar color="primary" dark>
			<v-btn icon @click="dialog = false">
				<v-icon>mdi-close</v-icon>
			</v-btn>
			<span>{{ editing ? $t("server.edit") : $t("server.add") }} MongoDB {{ $t("server.server") }}</span>
			<v-spacer></v-spacer>
        </v-toolbar>

        <v-card-text class="py-4">
			<v-form
			ref="form"
			v-model="valid"
			lazy-validation
			>
			<v-tabs v-model="tab">
				<v-tab value="tp" :disabled="editing&&tab==='ssh'">TCP/IP</v-tab>
				<v-tab value="ssh" :disabled="editing&&tab==='tp'">SSH</v-tab>
			</v-tabs>
			<v-tabs-window v-model="tab" class="mt-4">
				<v-tabs-window-item value="tp">
				    <v-text-field
						v-model="serverInfo.mongoHost"
						:label="$t('server.mongoHost')"
						:rules="hostRules"
						required
						variant="solo-filled"
						density="comfortable"
						:placeholder="$t('server.mongoHostPlaceholder')"
					></v-text-field>

					<v-text-field
						v-model.number="serverInfo.mongoPort"
						:label="$t('server.mongoPort')"
						:rules="portRules"
						required
						type="number"
						variant="solo-filled"
						class="mb-2"
						density="comfortable"
						:placeholder="$t('server.mongoPortPlaceholder')"
					></v-text-field>

					<v-radio-group
						v-model="serverInfo.authMethod"
						density="com"
						class="mb-2"
						size="small"
					>
						<template v-slot:label>
							<p class="text-body-2">{{ $t('server.authMethods') }}</p>
						</template>
						<v-radio
							value="none"
							class="mb-2"
						><template v-slot:label>
							<p class="text-body-2">{{ $t('server.noAuth') }}</p>
						</template></v-radio>
						<v-radio
							value="userpass"
						><template v-slot:label>
							<p class="text-body-2">{{ $t('server.authNeeded') }}</p>
						</template></v-radio>
					</v-radio-group>

					<v-text-field
						v-if="serverInfo.authMethod === 'userpass'"
						v-model="serverInfo.mongoUsername"
						:label="$t('server.mongoUserName')"
						:rules="serverInfo.authMethod === 'userpass' ? usernameRules : []"
						:required="serverInfo.authMethod === 'userpass'"
						class="mb-2"
						variant="solo-filled"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-if="serverInfo.authMethod === 'userpass'"
						v-model="serverInfo.mongoPassword"
						:label="$t('server.mongoPassWord')"
						:rules="serverInfo.authMethod === 'userpass' ? passwordRules : []"
						:required="serverInfo.authMethod === 'userpass'"
						type="password"
						class="mb-2"
						variant="solo-filled"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model="serverInfo.dbName"
						:label="$t('server.databaseName')"
						:rules="dbNameRules"
						required
						variant="solo-filled"
						density="comfortable"
					></v-text-field>
				</v-tabs-window-item>

				<v-tabs-window-item value="ssh">
					<v-text-field
						v-model="serverInfo.sshHost"
						:label="$t('server.sshHost')"
						:rules="serverInfo.type === 'ssh' ? hostRules : []"
						required
						variant="solo-filled"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model.number="serverInfo.sshPort"
						:label="$t('server.sshPort')"
						:rules="serverInfo.type === 'ssh' ? portRules : []"
						required
						type="number"
						variant="solo-filled"
						class="mb-2"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model="serverInfo.sshUsername"
						:label="$t('server.sshUserName')"
						:rules="serverInfo.type === 'ssh' ? usernameRules : []"
						required
						variant="solo-filled"
						class="mb-2"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model="serverInfo.sshPassword"
						:label="$t('server.sshPassWord')"
						:rules="serverInfo.type === 'ssh' ? passwordRules : []"
						required
						type="password"
						variant="solo-filled"
						class="mb-2"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model="serverInfo.mongoHost"
						disabled="true"
						:label="$t('server.mongoIntranetHost')"
						:rules="hostRules"
						required
						variant="solo-filled"
						density="comfortable"
						:placeholder="$t('server.mongoHostPlaceholder')"
					></v-text-field>

					<v-text-field
						v-model.number="serverInfo.mongoPort"
						:label="$t('server.mongoPort')"
						:rules="portRules"
						required
						type="number"
						variant="solo-filled"
						class="mb-2"
						density="comfortable"
						:placeholder="$t('server.mongoPortPlaceholder')"
					></v-text-field>

					<v-radio-group
						v-model="serverInfo.authMethod"
						density="com"
						class="mb-2"
						size="small"
					>
						<template v-slot:label>
							<p class="text-body-2">{{ $t('server.authMethods') }}</p>
						</template>
						<v-radio
							value="none"
							class="mb-2"
						><template v-slot:label>
							<p class="text-body-2">{{ $t('server.noAuth') }}</p>
						</template></v-radio>
						<v-radio
							value="userpass"
						><template v-slot:label>
							<p class="text-body-2">{{ $t('server.authNeeded') }}</p>
						</template></v-radio>
					</v-radio-group>

					<v-text-field
						v-if="serverInfo.authMethod === 'userpass'"
						v-model="serverInfo.mongoUsername"
						:label="$t('server.mongoUserName')"
						:rules="serverInfo.authMethod === 'userpass' ? usernameRules : []"
						:required="serverInfo.authMethod === 'userpass'"
						class="mb-2"
						variant="solo-filled"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-if="serverInfo.authMethod === 'userpass'"
						v-model="serverInfo.mongoPassword"
						:label="$t('server.mongoPassWord')"
						:rules="serverInfo.authMethod === 'userpass' ? passwordRules : []"
						:required="serverInfo.authMethod === 'userpass'"
						type="password"
						class="mb-2"
						variant="solo-filled"
						density="comfortable"
					></v-text-field>

					<v-text-field
						v-model="serverInfo.dbName"
						:label="$t('server.databaseName')"
						:rules="dbNameRules"
						required
						variant="solo-filled"
						density="comfortable"
					></v-text-field>
				</v-tabs-window-item>
			</v-tabs-window>
          </v-form>
        </v-card-text>

        <v-card-actions class="pa-4 bg-grey-50">
          <v-spacer></v-spacer>
          <v-btn
		  	v-if="!editing"
            color="primary"
            @click="resetForm"
            class="mr-2"
            width="100"
          >
            {{$t("server.reset") }}
          </v-btn>
          <v-btn
            color="primary"
            @click="submitForm"
            :disabled="!valid"
            width="100"
          >
            {{$t("server.save") }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

	<!-- 删除对话框 -->
    <v-dialog v-model="removeDialog" max-width="400" persistent>
      <v-card
        prepend-icon="mdi-lan-disconnect"
        text=""
        :title="$t('server.deleteServer')+'?'"
      >
        <template v-slot:actions>
          <v-spacer></v-spacer>

          <v-btn @click="removeDialog = false">
            {{$t("server.cancel") }}
          </v-btn>

          <v-btn @click="removeServer" color="error">
            {{$t("server.delete") }}
          </v-btn>
        </template>
      </v-card>
    </v-dialog>

	<!-- 提示对话框 -->
	<v-dialog
		class="text-body-2"
		v-model="ELDialog"
		max-width="400"
		persistent
    >
      <v-card>
	  	<template v-slot:title>
			<span class="font-weight-black" style="color: #0288D1">Information</span>
		</template>

	  	<v-card-text v-html="ELText" class="bg-surface-light pt-4">
		</v-card-text>
        <template v-slot:actions>
          <v-spacer></v-spacer>

			<!-- 如果取消不发送信号，后台使用once监听就不会自动销毁，导致内存泄漏 -->
          <v-btn @click="appWebview.emit('FirstKeyChecked', false);ELDialog = false">
            {{ $t("server.cancel") }}
          </v-btn>

          <v-btn @click="appWebview.emit('FirstKeyChecked', true);ELDialog = false" color="primary">
           {{ $t("server.confirm") }}
          </v-btn>
        </template>
      </v-card>
    </v-dialog>

	<!-- 警告对话框 -->
	<v-dialog v-model="WNDialog" width="auto">
      <v-card max-width="400" :text="$t('server.KeyCheckFailed')">
	  	<template v-slot:title>
			<span class="font-weight-black" style="color: #FFB300">Warning</span>
		</template>
        <template v-slot:actions>
          <v-btn
            class="ms-auto"
            text="Ok"
            @click="WNDialog = false"
          ></v-btn>
        </template>
      </v-card>
    </v-dialog>

  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { v4 as uuidv4 } from 'uuid';
import { getCurrentInstance } from 'vue'
import { warn, info, error } from '@tauri-apps/plugin-log';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';


export default {
  inject: ['showSnackbar'],
  data() {
    return {
	  editing: false,
      dialog: false,
	  removeDialog: false,
	  ELDialog: false,
	  ELText: '',
	  WNDialog: false,
	  appWebview: null,
	  index: 0,
	  editIndex: 0,
      valid: false,
	  tab: "tp",
      servers: [],
	  loadingStates: [],
	  connectedStates: [],
      // 当前正在编辑的服务器信息
      serverInfo: {
		id: '',
		type: 'tp',
		authMethod: 'none',
		sshHost: '',
		sshPort: 22,
		sshUsername: '',
		sshPassword: '',
        mongoHost: '127.0.0.1',
        mongoPort: 27017,
        mongoUsername: '',
        mongoPassword: '',
        dbName: 'test',
      },
	  
      // 表单验证规则
      hostRules: [
        v => !!v || this.$t('server.validRules.hostRules'),
      ],
      portRules: [
        v => !isNaN(Number(v)) || this.$t('server.validRules.portRules[0]'),
        v => (v >= 1 && v <= 65535) || this.$t('server.validRules.portRules[1]')
      ],
      usernameRules: [
        v => !!v || this.$t('server.validRules.userNameRules')
      ],
      passwordRules: [
        v => !!v || this.$t('server.validRules.passWordRules')
      ],
      dbNameRules: [
        v => !!v || this.$t('server.validRules.dbNameRules')
      ],
    }
  },

  watch: {
	tab: {
		handler(newVal, oldVal) {
			if(newVal === 'tp'){
				this.serverInfo.type = 'tp'
			}
			else if(newVal === 'ssh'){
				this.serverInfo.type = 'ssh'
			}
		},
	},
  },

  async created() {
		// 初始化 webview
		this.appWebview = getCurrentWebviewWindow();
		this.setupListeners();
	},

  mounted() {
	//引用类型
    let globalConnectedStates = getCurrentInstance().appContext.config.globalProperties.$globalConnectedStates;
	
	load('store.json', { autoSave: false })
	.then(store => {
		store.get('servers')
		.then(res => {
			for(let i = 0;i < res.length;i++) {
				this.loadingStates.push(false);
				if(globalConnectedStates.length < res.length){
					globalConnectedStates.push(false);
				}
			}
			this.servers = res
		})
	})
	this.connectedStates = globalConnectedStates;
  },

  methods: {
	setupListeners() {
		// 监听ssh第一次登录事件，需要持续监听
		this.appWebview.listen('FirstKeyCheck', (event) => {
			if (event.payload){
				this.ELText = this.$t('server.FirstKeyCheck.msg') + "<br>" + 
					this.$t('server.FirstKeyCheck.fp') + event.payload.fp + "<br>" + 
					this.$t('server.FirstKeyCheck.alg') + event.payload.alg;
				this.ELDialog = true;
			}
		});
	},

    // 提交表单，添加服务器
    async submitForm() {
      if (this.$refs.form.validate()) {
		if(!this.editing){
			this.serverInfo.id = uuidv4();
		}else{
			// 如果连接状态中编辑保存需提前终止连接
			if(this.connectedStates[this.editIndex] = true){
				this.disConnection(this.editIndex);
			}
		}
		const {sshPassword, mongoPassword, ...newServer} = this.serverInfo;
		// 认证信息加密
		if(this.serverInfo.authMethod === 'userpass' || this.serverInfo.type === 'ssh'){
			await invoke('mongo_data_encrypt', {password:this.serverInfo.id, plaintext:JSON.stringify(this.serverInfo)})
			.then(res => info(res))
			.catch(err => error(err))
		}
		else{
			if(this.editing){
				if(this.servers[this.editIndex].authMethod === 'userpass'){
					await invoke('mongo_delete_encrypt_data', {skey: this.serverInfo.id})
					.then(res => info(res))
					.catch(err => error(err))
				}
			}
		}
		
		const store = await load('store.json', { autoSave: false });
		const val = await store.get('servers');
		if (val && Array.isArray(val)) {
			if(!this.editing){
				val.push(newServer);
				await store.set('servers', val);
				this.servers = val;
			}else{
				if(val[this.editIndex].id == newServer.id){
					val[this.editIndex] = newServer;
					await store.set('servers', val);
					this.servers = val;
				}
				// 移出连接池
				await invoke('mongo_clear_connection', {id:newServer.id})
				.then(res => info(res))
				.catch(err => error(err));

				this.editIndex = 0;
				this.editing = false;
			}
			
		}else {
			await store.set('servers', [newServer]);
			this.servers = [newServer];
		}
		await store.save();
        
        // 关闭对话框并重置表单
        this.dialog = false;
        this.resetForm();
      }
    },
	async editServer(index) {
		// 使用深拷贝避免直接修改原数据
		this.serverInfo = JSON.parse(JSON.stringify(this.servers[index]));
		this.tab = this.serverInfo.type;
		this.editing = true;
		this.editIndex = index;
		this.dialog = true;
	},
    
    // 重置表单
    resetForm() {
      this.serverInfo = {
		id: '',
		type: 'tp',
		authMethod: 'none',
		sshHost: '',
		sshPort: 22,
		sshUsername: '',
		sshPassword: '',
        mongoHost: '127.0.0.1',
        mongoPort: 27017,
        mongoUsername: '',
        mongoPassword: '',
        dbName: 'test',
      };
      this.valid = false;
    },
    
    // 移除服务器
    removeServerCheck(index) {
		this.index = index;
		this.removeDialog = true;
    },
	async removeServer() {
		const store = await load('store.json', { autoSave: false });
		const removedServer = this.servers.splice(this.index, 1)[0];
		await store.set('servers', this.servers);
		await store.save();
		// 移出连接池
		await invoke('mongo_clear_connection', {id:removedServer.id})
		.then(res => info(res))
		.catch(err => error(err));

		// 如有认证销毁加密信息
		if(removedServer.authMethod === 'userpass' || removedServer.type === 'ssh'){
			await invoke('mongo_delete_encrypt_data', {skey:removedServer.id})
			.then(res => info(res))
			.catch(err => error(err));
		}

		this.removeDialog = false;
		warn(`${removedServer.id}: Service has been removed.`);
	},

	async disConnection(index) {
		await invoke('mongo_clear_connection', {id:this.servers[index].id})
		.then(res => info(res))
		.catch(err => error(err));
		this.connectedStates[index] = false;
	},
    
    // 测试连接
    testConnection(index) {
		const hasTrue = this.loadingStates.includes(true);
		if (hasTrue) return;

		this.loadingStates[index] = true;
		invoke('mongo_connect_server', this.servers[index])
		.then(res => {
			this.connectedStates[index] = true;
			this.showSnackbar(this.$t("server.connectSuccess"), 'success', 1500);

			this.$router.push(`/collections?server=${JSON.stringify(this.servers[index])}&collection_msg=${JSON.stringify(res)}`);
		})
		.catch(err => {
			if(err === "Code-3581"){
				return;
			}else if(err === "Code-3067"){
				this.WNDialog = true;
				return;
			}
			this.showSnackbar(err, 'error', 3000);
		})
		.finally(() => {
			this.loadingStates[index] = false;
		})
    },
  }
}
</script>