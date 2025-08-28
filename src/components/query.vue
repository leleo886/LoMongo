<template>
  <v-container fluid class="py-6">
    <v-row>
      <v-col
        cols="12"
        v-for="(item, index) in queryCommands"
        :key="index"
        class="pb-1"
      >
        <v-card elevation="3" class="hover" >
          <v-card-text>
            <p style="font-size: 0.9rem;">{{ item.description }}</p>
			<div class="d-flex justify-end">
				<v-btn 
					size="small"
					color="primary"
					variant="text"
					@click="copyToClipboard(item.query, index)"
					>
					<v-icon class="mr-1">{{ copyIndex === index ? 'mdi-check-bold' : 'mdi-content-copy'}}</v-icon>
				</v-btn>
			</div>
			
            <div class="code-block">
				<code>{{ item.query }}</code>
            </div>
          </v-card-text>

        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
export default {
  name: 'MongoDbQueryCards',
  data() {
    return {
		copyIndex: -1,
		queryCommands: [
			{
				description: this.$t("query.desc0"),
				query: '{_id: ObjectId("689ee2c8bv123456789bv123")}'
			},
			{
				description:this.$t("query.desc1"),
				query: '{ age: { $gt: 25 } }'
			},
			{
				description: this.$t("query.desc2"),
				query: '{ gender: "female" }'
			},
			{
				description: this.$t("query.desc3"),
				query: '{ datetime: {$gt: ISODate("2025-08-18T00:00:00.000Z")}}'
			},
			{
				description: this.$t("query.desc4"),
				query: '{accountBalance:{$lt: NumberDecimal("1250.55")}}'
			},
			{
				description: this.$t("query.desc5"),
				query: '{ status: { $ne: "active" } }'
			},
			{
				description: this.$t("query.desc6"),
				query: '{ hobbies: { $in: ["reading"] } }'
			},
			{
				description: this.$t("query.desc7"),
				query: '{ age: { $gte: 20, $lte: 30 } }'
			},
			{
				description: this.$t("query.desc8"),
				query: '{ email: { $exists: true } }'
			},
			{
				description: this.$t("query.desc9"),
				query: '{ username: { $regex: "^j", $options: "i"} }'
			},
			{
				description: this.$t("query.desc10"),
				query: '{ comments: { $size: 5 } }'
			},
			{
				description: this.$t("query.desc11"),
				query: '{ $and: [ { age: { $gt: 25 } }, { gender: "male" } ] }'
			}
		]
    };
  },
  methods: {

    // 复制到剪贴板
    async copyToClipboard(text, index) {
      try {
        await navigator.clipboard.writeText(text);
		this.copyIndex = index;
		setTimeout(() => {
			this.copyIndex = -1;
		}, 1000);
      } catch (err) {
        // 旧版本浏览器的处理
        const textArea = document.createElement('textarea');
        textArea.value = text;
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
      }
    },

  }
};
</script>

<style scoped>
.code-block {
  background-color: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 6px;
  padding: 12px;
  margin: 10px 0;
  overflow-x: auto;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  white-space: pre-wrap; 
  word-break: normal;
}

.code-block code {
  display: block;
  color: #2d3748;
  line-height: 1.5;
  
}
</style>