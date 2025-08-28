<template>
	<v-card 
		:title="server.dbName"
	></v-card>
	<v-divider :thickness="5"></v-divider>
 	 <v-card
	 	v-for="collection in collections"
		color="primary"
		variant="outlined"
		class="ma-3"
        :title="collection.collection"
		
      >
        <template v-slot:prepend>
          <v-icon color="primary" icon="mdi-file-multiple-outline"></v-icon>
        </template>
        <template v-slot:append>
			<v-btn icon="mdi-open-in-new" color="primary" variant="text" 
			:to="{
				path: '/coll',
				query: { collection: collection.collection,
					 server:JSON.stringify(this.server)
				 }
			}"
			></v-btn>
        </template>
        <v-card-text class="d-flex justify-space-around flex-wrap">
			<v-chip size="small" color="se" class="my-1">
				{{ $t("collection.storageSize") }}: {{ formattedSize(collection.storageSize) }}
			</v-chip>
			<v-chip size="small" color="primary" class="my-1">
				{{ $t("collection.docCount") }}: {{ collection.count }}
			</v-chip>
			<v-chip size="small" color="primary" class="my-1"
			prepend-icon="mdi-arrow-right-drop-circle-outline">
				{{ $t("collection.indexCount") }}: {{ collection.indexCount }}
				<v-tooltip
				activator="parent"
				location="bottom"
				><p v-for="name in collection.indexes">{{ name }}</p></v-tooltip>
			</v-chip>
		</v-card-text>
	</v-card>

	<div v-if="collections.length == 0">
      <v-card class="text-center elevation-1">
        <v-card-text class="py-10 px-4">
          <v-icon size="64" class="text-grey-400 mb-4" color="warning">mdi-rhombus-split</v-icon>
          <p class="text-h6 text-grey-500">{{ $t("collection.collectionNotFound") }}</p>
        </v-card-text>
      </v-card>
    </div>
</template>

<script>
	export default{
		data() {
			return {
				server:{},
				collections:[]
			}
		},
		mounted() {
			this.server = JSON.parse(this.$route.query.server);
			this.collections = JSON.parse(this.$route.query.collection_msg);
		},
		methods: {
			formattedSize(fileSizeInBytes) {
				const bytes = fileSizeInBytes;
				if (bytes === 0) return '0 Bytes';
				
				const k = 1024;
				const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
				const i = Math.floor(Math.log(bytes) / Math.log(k));
				
				return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
			}
		},
	}
</script>