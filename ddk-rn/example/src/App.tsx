import { StyleSheet, View, Text } from 'react-native';
import { version } from '@bennyhodl/ddk-rn';

export default function App() {
  return (
    <View style={styles.container}>
      <Text><code>@bennyhodl/ddk-rn version</code>: {version()}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  box: {
    width: 60,
    height: 60,
    marginVertical: 20,
  },
});
