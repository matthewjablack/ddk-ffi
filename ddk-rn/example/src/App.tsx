import { StyleSheet, View, Text } from 'react-native';
import { version } from '@bennyblader/ddk-rn';

export default function App() {
  return (
    <View style={styles.container}>
      <Text>@bennyhodl/ddk-rn version: {version()}</Text>
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
