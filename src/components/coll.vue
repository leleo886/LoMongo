<template>
  <v-card>
	<div class="my-2 d-flex justify-space-between align-center">
		<span class="px-5">{{server.dbName }} > {{ collection_name }} </span>
		<v-btn
			append-icon="mdi-arrow-top-right"
			variant="tonal"
			density="compact"
			color="primary"
			class="mx-4"
			:to="{ name: 'QueryExample' }"
			>
			{{ $t("query.commandTemplate") }}
			<template v-slot:append>
				<v-icon></v-icon>
			</template>
		</v-btn>
	</div>
	
	<v-card-title>
      <v-textarea 
	  	v-model="query"
        density="comfortable"
		clearable
        :placeholder="$t('collection.queryPlaceholder')+' { field : \'value\' }'"
		rows="1"
		max-rows="6"
		auto-grow
        variant="outlined"
		color="primary"
        single-line>
		<template v-slot:append-inner>
			<v-btn
				density="comfortable"
				icon="mdi-magnify"
				variant="text"
				@click="search()"
			></v-btn>
		</template>
	  </v-textarea>
    </v-card-title>
    <v-divider></v-divider>
	<div class="d-flex justify-space-around align-center flex-wrap my-2">
		<v-pagination
			v-model="currentPage"
			:length="totalPages"
			:total-visible="1"
			rounded="circle"
			active-color="primary"
			density="comfortable"
			size="small"
		></v-pagination>
		<v-chip color="primary" size="small" density="comfortable">
			{{totalItems == 0 ? 0 : (currentPage-1) * pageSize + 1}} - {{ currentPage * pageSize > totalItems ? totalItems : currentPage * pageSize}} of {{ totalItems }}
		</v-chip>
		<v-btn 
			icon="mdi-refresh" 
			density="comfortable" 
			color="primary" 
			size="small"
			@click="currentPage === 1 ? fetchData() : currentPage = 1" 
			variant="tonal">
		</v-btn>
		<v-btn
			color="primary"
			variant="outlined"
			append-icon="mdi-chevron-down"
			class="px-1"
			density="comfortable"
			size="small"
			>
			{{ pageSize }}
			<v-menu activator="parent">
				<v-list>
				<v-list-item
					v-for="(item, index) in [25,50,75,100]"
					:key="index"
					:value="index"
					@click="pageSize=item"
				>
					<v-list-item-title>{{ item }}</v-list-item-title>
				</v-list-item>
				</v-list>
			</v-menu>
		</v-btn>
	</div>
	<v-divider></v-divider>
  </v-card>

    <v-container max-height="70vh" class="pa-0 d-flex flex-column">
      <v-card  style="overflow-y: scroll;">
        <v-card-text>
          <div v-if="loading" class="text-center py-4">
            <v-progress-circular indeterminate color="primary"></v-progress-circular>
            <div class="mt-2">{{ $t("collection.loadingData") }}</div>
          </div>
          
          <div v-else>
            <div v-if="items.length > 0" class="document-list">
              <div v-for="(doc, index) in items" :key="index" class="document-item mb-4">
                <div class="document-content pa-3 grey lighten-5 rounded-b">
                  <vue-json-pretty
                    v-if="doc"
                    :data="transformMongoDBData(doc)"
                    :deep="1"
                    showLength
					showIcon
                  >
				  <template #renderNodeValue="{ node, defaultValue }">
					<span v-if="isMongoDBType(node)" >
						{{ formatMongoDBValue(node) }}
					</span>
					<span v-else>
						{{ defaultValue }}
					</span>
				</template>
				  </vue-json-pretty>
				  
                </div>
              </div>
            </div>
            
            <v-alert v-else type="info" class="mt-4">
              {{$t("collection.noAnyDocs")}}
            </v-alert>
          </div>
        </v-card-text>
      </v-card>
    </v-container>
</template>

<script>
import { invoke } from '@tauri-apps/api/core';
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

export default {
	inject: ['showSnackbar'],
	components: {
		VueJsonPretty
	},
	data() {
		return {
			query:'',
			server:{},
			collection_name:'',
			items: [],
			currentPage: 1,
			pageSize: 25,
			totalPages: 0,
			totalItems: 0,
			loading: false
		}
	},
	mounted() {
		this.server = JSON.parse(this.$route.query.server);
		this.collection_name = this.$route.query.collection;
		this.fetchData();
	},

	watch: {
		pageSize(newVal, oldVal) {
			if (newVal !== oldVal) {
				this.handlePageSizeChange();
			}
		},
		currentPage(newVal, oldVal) {
			if (newVal !== oldVal) {
				this.fetchData();
			}
		},
		query(newVal, oldVal) {
			if (newVal !== oldVal && newVal === "") {
				this.currentPage = 1;
				this.fetchData();
			}
		}
	},

	methods: {
		// 获取数据
		async fetchData() {
			this.loading = true;
			try {
				const result = await invoke('mongo_collection', {
					...this.server,
					collectionName: this.collection_name, 
					page: this.currentPage - 1,
					pageSize: this.pageSize,
					query: this.query
				});
				
				
				this.items = result.documents || [];
				this.totalItems = result.total_count || 0;
				this.totalPages = result.total_pages || Math.ceil(this.totalItems / this.pageSize);
				
			} catch (error) {
				this.showSnackbar("Error!", 'red', 1000);
				this.items = [];
				this.totalPages = 0;
				this.totalItems = 0;
			} finally {
				this.loading = false;
			}
		},
		search(){
			this.currentPage = 1;
			this.fetchData();
		},
    // 安全地处理 MongoDB 数据
    transformMongoDBData(data) {
      try {
        // 处理数组
        if (Array.isArray(data)) {
          return data.map(item => this.transformMongoDBData(item));
        }
        
        // 处理对象
        if (data && typeof data === 'object') {
          // 检查是否为 MongoDB 扩展 JSON 格式
          const extendedJSON = this.processExtendedJSON(data);
          if (extendedJSON.processed) {
            return extendedJSON.value;
          }
          
          // 递归处理普通对象
          const result = {};
          for (const [key, value] of Object.entries(data)) {
            result[key] = this.transformMongoDBData(value);
          }
          return result;
        }
        
        // 返回原始值
        return data;
      } catch (error) {
        console.warn('Error transforming MongoDB data:', error, data);
        // 返回原始数据，确保程序继续运行
        return data;
      }
    },
    
    // 安全地处理 MongoDB 扩展 JSON 格式
    processExtendedJSON(data) {
      try {
		// 处理 ObjectId
		if (data.$oid) {
			return { processed: true, value: this.formatObjectId(data.$oid) };
		}
		
		// 处理日期 - 支持多种格式
		if (data.$date) {
			let dateValue;
			if (typeof data.$date === 'string') {
				// 标准 ISO 字符串格式
				dateValue = this.formatISODate(data.$date);
			} else if (data.$date && typeof data.$date === 'object') {
				// 处理嵌套格式
				if (data.$date.$numberLong) {
					const timestamp = parseInt(data.$date.$numberLong, 10);
					dateValue = new Date(timestamp).toISOString();
				} else if (data.$date.$numberInt) {
					const timestamp = parseInt(data.$date.$numberInt, 10);
					dateValue = new Date(timestamp).toISOString();
				} else {
					// 未知格式，返回原始值
					dateValue = JSON.stringify(data.$date);
				}
			} else {
				// 未知格式，返回原始值
				dateValue = JSON.stringify(data.$date);
			}
			return { processed: true, value: dateValue };
		}
		
		// 处理时间戳
		if (data.$timestamp) {
			let t, i;
		
			if (data.$timestamp.t && typeof data.$timestamp.t === 'object') {
				if (data.$timestamp.t.$numberInt) {
					t = parseInt(data.$timestamp.t.$numberInt, 10);
				} else if (data.$timestamp.t.$numberLong) {
					t = parseInt(data.$timestamp.t.$numberLong, 10);
				} else {
					t = data.$timestamp.t;
				}
			} else {
				t = data.$timestamp.t;
			}
		
			if (data.$timestamp.i && typeof data.$timestamp.i === 'object') {
				if (data.$timestamp.i.$numberInt) {
					i = parseInt(data.$timestamp.i.$numberInt, 10);
				} else if (data.$timestamp.i.$numberLong) {
					i = parseInt(data.$timestamp.i.$numberLong, 10);
				} else {
					i = data.$timestamp.i;
				}
			} else {
				i = data.$timestamp.i;
			}
		
			return { processed: true, value: this.formatTimestamp({ t, i }) };
		}
		
		// 处理 Decimal128
		if (data.$numberDecimal) {
			return { processed: true, value: this.formatNumberDecimal(data.$numberDecimal) };
		}
		
		// 处理 Symbol
		if (data.$symbol) {
			return { processed: true, value: this.formatSymbol(data.$symbol) };
		}
		
		// 处理正则表达式
		if (data.$regularExpression) {
			return { processed: true, value: this.formatRegExp(data.$regularExpression) };
		}
		
		// 处理二进制数据
		if (data.$binary) {
			return { processed: true, value: this.formatBinary(data.$binary) };
		}
		
		// 处理 32 位整数
		if (data.$numberInt) {
			return { processed: true, value: parseInt(data.$numberInt, 10) };
		}
		
		// 处理 64 位整数
		if (data.$numberLong) {
			return { processed: true, value: parseInt(data.$numberLong, 10) };
		}
		
		// 处理双精度浮点数
		if (data.$numberDouble) {
			return { processed: true, value: parseFloat(data.$numberDouble) };
		}
		
		// 处理未定义
		if (data.$undefined !== undefined) {
			return { processed: true, value: 'undefined' };
		}
		
		// 处理最小值
		if (data.$minKey !== undefined) {
			return { processed: true, value: 'MinKey' };
		}
		
		// 处理最大值
		if (data.$maxKey !== undefined) {
			return { processed: true, value: 'MaxKey' };
		}
		
		// 不是扩展 JSON 格式
		return { processed: false, value: null };
      } catch (error) {
        console.warn('Error processing extended JSON:', error, data);
        // 遇到错误时返回原始数据
        return { processed: false, value: null };
      }
    },
    
    handlePageSizeChange() {
      this.currentPage = 1;
      this.fetchData();
    },
    
    formatObjectId(oid) {
      return `ObjectId('${oid}')`;
    },

    formatISODate(dateString) {
      try {
        const date = new Date(dateString);
        return date.toISOString();
      } catch (error) {
        console.warn('Error formatting date:', error, dateString);
        return `Invalid Date: ${dateString}`;
      }
    },

    formatTimestamp(ts) {
      return `Timestamp({ t: ${ts.t}, i: ${ts.i} })`;
    },

    formatNumberDecimal(value) {
      return value;
    },

    formatSymbol(symbol) {
      return `Symbol('${symbol}')`;
    },

    formatRegExp(regex) {
      try {
        const { pattern, options } = regex;
        return `/${pattern}/${options}`;
      } catch (error) {
        console.warn('Error formatting regex:', error, regex);
        return `Invalid Regex: ${JSON.stringify(regex)}`;
      }
    },

    formatBinary(binary) {
      try {
        return `Binary.createFromBase64('${binary.base64}', ${parseInt(binary.subType, 16)})`;
      } catch (error) {
        console.warn('Error formatting binary:', error, binary);
        return `Invalid Binary: ${JSON.stringify(binary)}`;
      }
    },

    isMongoDBType(node) {
      try {
        // 检查是否是格式化后的 MongoDB 类型
        const value = node.value;
        return typeof value === 'string' && (
          value.startsWith('ObjectId(') ||
          value.startsWith('Timestamp(') ||
          value.startsWith('Symbol(') ||
          value.startsWith('Binary.createFromBase64(') ||
          value.includes('T') && value.endsWith('Z') || // ISO 日期
          (value.startsWith('/') && value.includes('/')) // 正则表达式
        );
      } catch (error) {
        console.warn('Error checking MongoDB type:', error, node);
        return false;
      }
    },

    formatMongoDBValue(node) {
      return node.value;
    },
  }
}
</script>
<style>
.v-field__input{
	font-size: 14px;
}

.document-item {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
} 

.document-header {
  border-bottom: 1px solid #e0e0e0;
}

.vjs-tree-brackets{
	color: #ff8000;
}
.vjs-tree {
	max-height: 400px;
	overflow: auto;
  	font-family: "Source Code Pro", "DejaVu Sans Mono", "Liberation Mono", monospace;
  	font-size: 12px;
}
.vjs-value {
	white-space: nowrap;
}

</style>