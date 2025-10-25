<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue'
import * as echarts from 'echarts/core'
import { MapChart } from 'echarts/charts'
import { TooltipComponent, VisualMapComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import worldMapData from '../assets/world.json';
import { useRoute, useRouter } from 'vue-router'
import { useDark } from '@vueuse/core'
import { ApolloClient, gql, ServerError } from '@apollo/client'
import { setAuthProp } from '../auth'
import { HttpLink } from '@apollo/client'
import { InMemoryCache } from '@apollo/client'
import type { LinkClicks, LinkCountryClicks } from '../api'
import TitleBar from '../components/TitleBar.vue'

const route = useRoute()
const router = useRouter();
const darkTheme = useDark();
const linkId = parseInt(route.params.id as string ?? 0);

echarts.use([MapChart, TooltipComponent, VisualMapComponent, CanvasRenderer]);
echarts.registerMap('world', worldMapData as any);

const chartRef = ref<HTMLDivElement | null>(null);
let chartInstance: echarts.ECharts | null = null;

type LinkMetadata = {
  fetchLinkClicks: LinkClicks | null,
  fetchLinkCountryClicks: LinkCountryClicks[]
};

const metadata: Ref<LinkMetadata | null> = ref(null);

const client = new ApolloClient({
  link: new HttpLink({ uri: "/api" }),
  cache: new InMemoryCache(),
});

const fetchData = async () => {
  try {
    const res = await client.query({
      query: gql`
        query fetchLinkData($linkId: Int!) {
          fetchLinkClicks(linkId: $linkId) {
            clicks
          }
          fetchLinkCountryClicks(linkId: $linkId) {
            country
            clicks
          }
        }
      `,
      variables: {
        linkId: linkId
      }
    });

    metadata.value = res.data as LinkMetadata;
  } catch (err: any) {
    if (ServerError.is(err)) {
      if (err.statusCode == 401) {
        setAuthProp(null);
        router.push("/login");
      }

      console.error(`api call failed: ${err.message}`);
    } else {
      console.error(`api call failed: ${err.message}`);
    }
  }
};

const initGraph = () => {
  if (!chartRef.value || !metadata.value) {
    return;
  }

  chartInstance = echarts.init(chartRef.value);

  const echartsData = metadata.value.fetchLinkCountryClicks.map((value) => {
    return {
      name: value.country,
      value: value.clicks
    }
  });

  const chartOptions: echarts.EChartsCoreOption = {
    backgroundColor: darkTheme ? "#0" : '#ffffff',
    tooltip: { trigger: 'item', formatter: '{b}<br/>Clicks: {c}' },
    visualMap: {
      min: 0,
      max: 100,
      inRange: { color: ['#a0c4ff', '#023e8a'] },
      textStyle: { color: '#fff' },
      bottom: 20,
      left: 20,
    },
    series: [
      {
        name: 'World Heatmap',
        type: 'map',
        map: 'world',
        roam: true,
        emphasis: { label: { show: false } },
        data: echartsData,
      },
    ],
  };

  chartInstance.setOption(chartOptions);

  window.addEventListener('resize', () => chartInstance?.resize());
};

const deleteLink = async () => {
  try {
    await client.mutate({
      mutation: gql`
      mutation deleteLink($linkId: Int!) {
        deleteLink(linkId: $linkId)
      }
    `,
      variables: {
        linkId: linkId
      }
    });

    router.push("/dashboard");
  } catch (err: any) {
    if (ServerError.is(err)) {
      const serverError = err as ServerError;

      if (serverError.statusCode == 401) {
        setAuthProp(null);
        router.push("/login");
      }
    }

    console.error(`api call failed: ${err}`);
  }
}

onMounted(async () => {
  await fetchData();
  initGraph();
})

onBeforeUnmount(() => {
  chartInstance?.dispose();
});
</script>

<template>
  <TitleBar />
  <div class="py-8 flex justify-center">
    <div class="w-[800px] flex flex-col gap-6">
      <div class="flex flex-row">
        <p class="abosule w-full text-lg">Total click/s: {{ metadata?.fetchLinkClicks?.clicks ?? "N/A" }}</p>
        <div class="flex-auto"></div>
        <button @click="deleteLink" class="bg-red-900 hover:bg-red-800 text-white px-3 py-1 rounded text-sm">
          Delete
        </button>
      </div>
      <div class="flex flex-col items-center">
        <div ref="chartRef" class="w-full max-w-5xl h-[300px]"></div>
      </div>
      <div class="w-full h-[230px] overflow-auto">
        <table v-if="metadata?.fetchLinkCountryClicks && metadata.fetchLinkCountryClicks.length > 0" class="w-full">
          <thead>
            <tr class="bg-blue-950">
              <th class="px-4 py-2 border-b text-left w-full">Country</th>
              <th class="px-4 py-2 border-b text-left">Clicks</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="meta in metadata.fetchLinkCountryClicks" :key="meta.country" class="hover:bg-gray-950">
              <td class="px-4 py-2 border-b">{{ meta.country }}</td>
              <td class="px-4 py-2 border-b">{{ meta.clicks }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else>
          <p class="text-center">No clicks for now.</p>
        </div>
      </div>
    </div>
  </div>
</template>
